import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { CollateralVault } from "../target/types/collateral_vault";
import { 
  TOKEN_PROGRAM_ID, 
  ASSOCIATED_TOKEN_PROGRAM_ID,
  MINT_SIZE,
  getAssociatedTokenAddress,
  createAssociatedTokenAccountInstruction,
  createInitializeMintInstruction,
  createMintToInstruction,
  getMinimumBalanceForRentExemptMint,
} from "@solana/spl-token";
import { 
  PublicKey, 
  Keypair, 
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
  Transaction,
} from "@solana/web3.js";

describe("collateral-vault", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.CollateralVault as Program<CollateralVault>;
  const admin = provider.wallet;
  const user = Keypair.generate();
  
  let mint: Keypair;
  let mintPubkey: PublicKey;
  let vaultAuthority: PublicKey;
  let vaultAuthorityBump: number;

  before(async () => {
    // Airdrop SOL to user and admin
    const userAirdrop = await provider.connection.requestAirdrop(
      user.publicKey,
      2 * anchor.web3.LAMPORTS_PER_SOL
    );
    await provider.connection.confirmTransaction(userAirdrop);

    const adminAirdrop = await provider.connection.requestAirdrop(
      admin.publicKey,
      2 * anchor.web3.LAMPORTS_PER_SOL
    );
    await provider.connection.confirmTransaction(adminAirdrop);

    // Create test mint
    mint = Keypair.generate();
    mintPubkey = mint.publicKey;
    
    const mintRent = await getMinimumBalanceForRentExemptMint(provider.connection);
    
    const createMintTx = new Transaction().add(
      SystemProgram.createAccount({
        fromPubkey: admin.publicKey,
        newAccountPubkey: mintPubkey,
        space: MINT_SIZE,
        lamports: mintRent,
        programId: TOKEN_PROGRAM_ID,
      }),
      createInitializeMintInstruction(
        mintPubkey,
        6, // 6 decimals like USDT
        admin.publicKey,
        null // No freeze authority
      )
    );
    
    const { blockhash } = await provider.connection.getLatestBlockhash();
    createMintTx.feePayer = admin.publicKey;
    createMintTx.recentBlockhash = blockhash;
    createMintTx.sign(admin.payer, mint);
    const sig = await provider.connection.sendRawTransaction(createMintTx.serialize());
    await provider.connection.confirmTransaction(sig);

    // Initialize vault authority
    [vaultAuthority, vaultAuthorityBump] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault_authority")],
      program.programId
    );
  });

  it("Initializes vault authority", async () => {
    const authorizedPrograms = [program.programId]; // For testing

    // Check if vault authority already exists
    try {
      const existing = await program.account.vaultAuthority.fetch(vaultAuthority);
      console.log("Vault authority already initialized:", vaultAuthority.toString());
      return; // Skip if already initialized
    } catch (err) {
      // Account doesn't exist, proceed with initialization
    }

    const tx = await program.methods
      .initializeVaultAuthority(authorizedPrograms)
      .accounts({
        admin: admin.publicKey,
        vaultAuthority: vaultAuthority,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    console.log("Vault authority initialized:", tx);
  });

  it("Initializes user vault", async () => {
    const [vaultPda, vaultBump] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), user.publicKey.toBuffer()],
      program.programId
    );

    const vaultTokenAccount = await getAssociatedTokenAddress(
      mintPubkey,
      vaultPda,
      true
    );

    const [vaultAuthorityPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), user.publicKey.toBuffer()],
      program.programId
    );

    try {
      const tx = await program.methods
        .initializeVault()
        .accounts({
          user: user.publicKey,
          vault: vaultPda,
          vaultTokenAccount: vaultTokenAccount,
          mint: mintPubkey,
          vaultAuthorityPda: vaultAuthorityPda,
          vaultAuthority: vaultAuthority,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([user])
        .rpc();

      console.log("Vault initialized:", tx);

      const vaultAccount = await program.account.collateralVault.fetch(vaultPda);
      console.log("Vault account:", vaultAccount);
      
      // Verify vault was initialized correctly
      const chai = require("chai");
      chai.assert.equal(vaultAccount.owner.toString(), user.publicKey.toString(), "Owner should match user");
      chai.assert.equal(vaultAccount.totalBalance.toNumber(), 0, "Initial balance should be 0");
      chai.assert.equal(vaultAccount.lockedBalance.toNumber(), 0, "Initial locked balance should be 0");
      chai.assert.equal(vaultAccount.availableBalance.toNumber(), 0, "Initial available balance should be 0");
    } catch (err) {
      console.error("Error initializing vault:", err);
      throw err;
    }
  });

  it("Deposits collateral", async () => {
    const [vaultPda, vaultBump] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), user.publicKey.toBuffer()],
      program.programId
    );

    const vaultTokenAccount = await getAssociatedTokenAddress(
      mintPubkey,
      vaultPda,
      true
    );

    const userTokenAccount = await getAssociatedTokenAddress(
      mintPubkey,
      user.publicKey
    );

    const [vaultAuthorityPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), user.publicKey.toBuffer()],
      program.programId
    );

    // Create user token account if it doesn't exist
    const userTokenAccountInfo = await provider.connection.getAccountInfo(userTokenAccount);
    if (!userTokenAccountInfo) {
      const createATA = new Transaction().add(
        createAssociatedTokenAccountInstruction(
          user.publicKey,
          userTokenAccount,
          user.publicKey,
          mintPubkey
        )
      );
      await provider.sendAndConfirm(createATA, [user]);
    }

    // Mint tokens to user
    const amount = new anchor.BN(1000000); // 1 USDT (6 decimals)
    const mintTx = new Transaction().add(
      createMintToInstruction(
        mintPubkey,
        userTokenAccount,
        admin.publicKey,
        amount.toNumber()
      )
    );
    const { blockhash: mintBlockhash } = await provider.connection.getLatestBlockhash();
    mintTx.feePayer = admin.publicKey;
    mintTx.recentBlockhash = mintBlockhash;
    mintTx.sign(admin.payer);
    const mintSig = await provider.connection.sendRawTransaction(mintTx.serialize());
    await provider.connection.confirmTransaction(mintSig);

    try {
      const tx = await program.methods
        .deposit(amount)
        .accounts({
          user: user.publicKey,
          vault: vaultPda,
          userTokenAccount: userTokenAccount,
          vaultTokenAccount: vaultTokenAccount,
          mint: mintPubkey,
          vaultAuthorityPda: vaultAuthorityPda,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([user])
        .rpc();

      console.log("Deposit transaction:", tx);

      const vaultAccount = await program.account.collateralVault.fetch(vaultPda);
      console.log("Vault after deposit:", vaultAccount);
      
      // Verify deposit was successful
      const chai = require("chai");
      chai.assert.equal(vaultAccount.totalBalance.toNumber(), 1000000, "Total balance should be 1000000");
      chai.assert.equal(vaultAccount.availableBalance.toNumber(), 1000000, "Available balance should be 1000000");
      chai.assert.equal(vaultAccount.totalDeposited.toNumber(), 1000000, "Total deposited should be 1000000");
    } catch (err) {
      console.error("Error depositing:", err);
      throw err;
    }
  });

  it("Withdraws collateral", async () => {
    const [vaultPda, vaultBump] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), user.publicKey.toBuffer()],
      program.programId
    );

    const vaultTokenAccount = await getAssociatedTokenAddress(
      mintPubkey,
      vaultPda,
      true
    );

    const userTokenAccount = await getAssociatedTokenAddress(
      mintPubkey,
      user.publicKey
    );

    const [vaultAuthorityPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), user.publicKey.toBuffer()],
      program.programId
    );

    const amount = new anchor.BN(500000); // 0.5 USDT

    try {
      const tx = await program.methods
        .withdraw(amount)
        .accounts({
          user: user.publicKey,
          vault: vaultPda,
          userTokenAccount: userTokenAccount,
          vaultTokenAccount: vaultTokenAccount,
          mint: mintPubkey,
          vaultAuthorityPda: vaultAuthorityPda,
          vaultAuthority: vaultAuthority,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([user])
        .rpc();

      console.log("Withdraw transaction:", tx);

      const vaultAccount = await program.account.collateralVault.fetch(vaultPda);
      console.log("Vault after withdraw:", vaultAccount);
      
      // Verify withdraw was successful
      const chai = require("chai");
      chai.assert.equal(vaultAccount.totalBalance.toNumber(), 500000, "Total balance should be 500000 after withdraw");
      chai.assert.equal(vaultAccount.availableBalance.toNumber(), 500000, "Available balance should be 500000");
      chai.assert.equal(vaultAccount.totalWithdrawn.toNumber(), 500000, "Total withdrawn should be 500000");
    } catch (err) {
      console.error("Error withdrawing:", err);
      throw err;
    }
  });
});
