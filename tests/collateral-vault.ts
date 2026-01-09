import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { CollateralVault } from "../target/types/collateral_vault";
import { 
  TOKEN_PROGRAM_ID, 
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAssociatedTokenAddress,
  createAssociatedTokenAccountInstruction,
} from "@solana/spl-token";
import { 
  PublicKey, 
  Keypair, 
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
} from "@solana/web3.js";

describe("collateral-vault", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.CollateralVault as Program<CollateralVault>;
  const admin = provider.wallet;
  const user = Keypair.generate();
  
  // USDT mint (devnet)
  const mint = new PublicKey("Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB");
  
  let vaultAuthority: PublicKey;
  let vaultAuthorityBump: number;

  before(async () => {
    // Airdrop SOL to user
    const signature = await provider.connection.requestAirdrop(
      user.publicKey,
      2 * anchor.web3.LAMPORTS_PER_SOL
    );
    await provider.connection.confirmTransaction(signature);

    // Initialize vault authority
    [vaultAuthority, vaultAuthorityBump] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault_authority")],
      program.programId
    );
  });

  it("Initializes vault authority", async () => {
    const authorizedPrograms = [program.programId]; // For testing

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
    const [vaultPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), user.publicKey.toBuffer()],
      program.programId
    );

    const vaultTokenAccount = await getAssociatedTokenAddress(
      mint,
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
          mint: mint,
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
    } catch (err) {
      console.error("Error initializing vault:", err);
      throw err;
    }
  });

  it("Deposits collateral", async () => {
    const [vaultPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), user.publicKey.toBuffer()],
      program.programId
    );

    const vaultTokenAccount = await getAssociatedTokenAddress(
      mint,
      vaultPda,
      true
    );

    const userTokenAccount = await getAssociatedTokenAddress(
      mint,
      user.publicKey
    );

    const [vaultAuthorityPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), user.publicKey.toBuffer()],
      program.programId
    );

    const amount = new anchor.BN(1000000); // 1 USDT (6 decimals)

    try {
      const tx = await program.methods
        .deposit(amount)
        .accounts({
          user: user.publicKey,
          vault: vaultPda,
          userTokenAccount: userTokenAccount,
          vaultTokenAccount: vaultTokenAccount,
          mint: mint,
          vaultAuthorityPda: vaultAuthorityPda,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([user])
        .rpc();

      console.log("Deposit transaction:", tx);

      const vaultAccount = await program.account.collateralVault.fetch(vaultPda);
      console.log("Vault after deposit:", vaultAccount);
    } catch (err) {
      console.error("Error depositing:", err);
      throw err;
    }
  });

  it("Withdraws collateral", async () => {
    const [vaultPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), user.publicKey.toBuffer()],
      program.programId
    );

    const vaultTokenAccount = await getAssociatedTokenAddress(
      mint,
      vaultPda,
      true
    );

    const userTokenAccount = await getAssociatedTokenAddress(
      mint,
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
          mint: mint,
          vaultAuthorityPda: vaultAuthorityPda,
          vaultAuthority: vaultAuthority,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([user])
        .rpc();

      console.log("Withdraw transaction:", tx);

      const vaultAccount = await program.account.collateralVault.fetch(vaultPda);
      console.log("Vault after withdraw:", vaultAccount);
    } catch (err) {
      console.error("Error withdrawing:", err);
      throw err;
    }
  });
});
