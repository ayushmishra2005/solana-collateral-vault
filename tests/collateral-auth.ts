import * as anchor from "@coral-xyz/anchor";
import { BN, Program } from "@coral-xyz/anchor";
import { CollateralVault } from "../target/types/collateral_vault";
import { TestCaller } from "../target/types/test_caller";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
  createAssociatedTokenAccount,
  createMint,
  getAssociatedTokenAddress,
  mintTo,
  transfer,
} from "@solana/spl-token";
import { Keypair, PublicKey, SystemProgram } from "@solana/web3.js";
import { assert } from "chai";

const BPF_UPGRADEABLE_LOADER = new PublicKey(
  "BPFLoaderUpgradeab1e11111111111111111111111"
);

function programDataPda(programId: PublicKey): PublicKey {
  return PublicKey.findProgramAddressSync(
    [programId.toBuffer()],
    BPF_UPGRADEABLE_LOADER
  )[0];
}

describe("vault authority bootstrap", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.CollateralVault as Program<CollateralVault>;
  const testCaller = anchor.workspace.TestCaller as Program<TestCaller>;
  const upgradeAuthority = (provider.wallet as anchor.Wallet).payer;
  const attacker = Keypair.generate();

  let vaultAuthority: PublicKey;

  async function expectFail(fn: () => Promise<unknown>) {
    try {
      await fn();
    } catch (err: any) {
      if (err?.message === "should have failed") {
        throw err;
      }
      return err;
    }
    throw new Error("should have failed");
  }

  before(async () => {
    const sig = await provider.connection.requestAirdrop(
      attacker.publicKey,
      2 * anchor.web3.LAMPORTS_PER_SOL
    );
    await provider.connection.confirmTransaction(sig);

    [vaultAuthority] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault_authority")],
      program.programId
    );
  });

  function authorityAccounts(signer: PublicKey) {
    return {
      upgradeAuthority: signer,
      program: program.programId,
      programData: programDataPda(program.programId),
      vaultAuthority,
      systemProgram: SystemProgram.programId,
    };
  }

  function extraProgramIds(n: number): PublicKey[] {
    return Array.from({ length: n }, () => Keypair.generate().publicKey);
  }

  it("rejects init from an unrelated signer", async () => {
    await expectFail(() =>
      program.methods
        .initializeVaultAuthority([attacker.publicKey])
        .accounts(authorityAccounts(attacker.publicKey))
        .signers([attacker])
        .rpc()
    );
  });

  it("rejects more than the max authorized programs", async () => {
    const tooMany = extraProgramIds(11);
    await expectFail(() =>
      program.methods
        .initializeVaultAuthority(tooMany)
        .accounts(authorityAccounts(upgradeAuthority.publicKey))
        .rpc()
    );
  });

  it("rejects duplicate authorized programs", async () => {
    await expectFail(() =>
      program.methods
        .initializeVaultAuthority([testCaller.programId, testCaller.programId])
        .accounts(authorityAccounts(upgradeAuthority.publicKey))
        .rpc()
    );
  });

  it("initializes the max authorized program list", async () => {
    const programs = [testCaller.programId, ...extraProgramIds(9)];
    await program.methods
      .initializeVaultAuthority(programs)
      .accounts(authorityAccounts(upgradeAuthority.publicKey))
      .rpc();

    const authority = await program.account.vaultAuthority.fetch(vaultAuthority);
    assert.equal(
      authority.admin.toString(),
      upgradeAuthority.publicKey.toString()
    );
    assert.equal(authority.authorizedPrograms.length, 10);
    assert.equal(
      authority.authorizedPrograms[0].toString(),
      testCaller.programId.toString()
    );
  });

  it("rejects a second initialization", async () => {
    await expectFail(() =>
      program.methods
        .initializeVaultAuthority([testCaller.programId])
        .accounts(authorityAccounts(upgradeAuthority.publicKey))
        .rpc()
    );
  });
});

describe("collateral authorization", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.CollateralVault as Program<CollateralVault>;
  const testCaller = anchor.workspace.TestCaller as Program<TestCaller>;
  const admin = (provider.wallet as anchor.Wallet).payer;

  const userA = Keypair.generate();
  const userB = Keypair.generate();
  const userC = Keypair.generate();
  const userD = Keypair.generate();

  let mint: PublicKey;
  let vaultAuthority: PublicKey;

  function vaultPda(owner: PublicKey): PublicKey {
    return PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), owner.toBuffer()],
      program.programId
    )[0];
  }

  function cpiAuthority(): PublicKey {
    return PublicKey.findProgramAddressSync(
      [Buffer.from("cpi_authority")],
      testCaller.programId
    )[0];
  }

  async function vaultToken(owner: PublicKey): Promise<PublicKey> {
    return getAssociatedTokenAddress(mint, vaultPda(owner), true);
  }

  async function userToken(owner: PublicKey): Promise<PublicKey> {
    return getAssociatedTokenAddress(mint, owner);
  }

  async function fetchVault(owner: PublicKey) {
    return program.account.collateralVault.fetch(vaultPda(owner));
  }

  function assertSplit(vault: {
    totalBalance: BN;
    availableBalance: BN;
    lockedBalance: BN;
  }) {
    assert.equal(
      vault.totalBalance.toNumber(),
      vault.availableBalance.toNumber() + vault.lockedBalance.toNumber()
    );
  }

  async function tokenAmount(ata: PublicKey): Promise<number> {
    const info = await provider.connection.getTokenAccountBalance(ata);
    return Number(info.value.amount);
  }

  async function assertVaultTokens(owner: PublicKey) {
    const vault = await fetchVault(owner);
    assertSplit(vault);
    const amount = await tokenAmount(await vaultToken(owner));
    assert.equal(amount, vault.totalBalance.toNumber());
  }

  async function expectFail(fn: () => Promise<unknown>) {
    try {
      await fn();
    } catch (err: any) {
      if (err?.message === "should have failed") {
        throw err;
      }
      return err;
    }
    throw new Error("should have failed");
  }

  async function airdrop(pubkey: PublicKey) {
    const sig = await provider.connection.requestAirdrop(
      pubkey,
      2 * anchor.web3.LAMPORTS_PER_SOL
    );
    await provider.connection.confirmTransaction(sig);
  }

  async function initVault(user: Keypair) {
    await program.methods
      .initializeVault()
      .accounts({
        user: user.publicKey,
        vault: vaultPda(user.publicKey),
        vaultTokenAccount: await vaultToken(user.publicKey),
        mint,
        vaultAuthorityPda: vaultPda(user.publicKey),
        vaultAuthority,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([user])
      .rpc();
  }

  async function deposit(user: Keypair, amount: number) {
    await program.methods
      .deposit(new BN(amount))
      .accounts({
        user: user.publicKey,
        vault: vaultPda(user.publicKey),
        userTokenAccount: await userToken(user.publicKey),
        vaultTokenAccount: await vaultToken(user.publicKey),
        mint,
        vaultAuthority: vaultPda(user.publicKey),
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([user])
      .rpc();
  }

  async function withdraw(user: Keypair, amount: number) {
    await program.methods
      .withdraw(new BN(amount))
      .accounts({
        user: user.publicKey,
        vault: vaultPda(user.publicKey),
        userTokenAccount: await userToken(user.publicKey),
        vaultTokenAccount: await vaultToken(user.publicKey),
        mint,
        vaultAuthorityPda: vaultPda(user.publicKey),
        vaultAuthority,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([user])
      .rpc();
  }

  async function lockCpi(owner: PublicKey, amount: number) {
    await testCaller.methods
      .lock(new BN(amount))
      .accounts({
        vault: vaultPda(owner),
        vaultAuthority,
        callerProgram: testCaller.programId,
        cpiAuthority: cpiAuthority(),
        vaultProgram: program.programId,
      })
      .rpc();
  }

  async function unlockCpi(owner: PublicKey, amount: number) {
    await testCaller.methods
      .unlock(new BN(amount))
      .accounts({
        vault: vaultPda(owner),
        vaultAuthority,
        callerProgram: testCaller.programId,
        cpiAuthority: cpiAuthority(),
        vaultProgram: program.programId,
      })
      .rpc();
  }

  async function transferCpi(from: PublicKey, to: PublicKey, amount: number) {
    await testCaller.methods
      .transfer(new BN(amount))
      .accounts({
        fromVault: vaultPda(from),
        toVault: vaultPda(to),
        fromVaultTokenAccount: await vaultToken(from),
        toVaultTokenAccount: await vaultToken(to),
        mint,
        toVaultAuthority: vaultPda(to),
        fromVaultAuthority: vaultPda(from),
        vaultAuthority,
        callerProgram: testCaller.programId,
        cpiAuthority: cpiAuthority(),
        tokenProgram: TOKEN_PROGRAM_ID,
        vaultProgram: program.programId,
      })
      .rpc();
  }

  async function lockDirect(owner: PublicKey, amount: number, callerProgram: PublicKey) {
    const [authority] = PublicKey.findProgramAddressSync(
      [Buffer.from("cpi_authority")],
      callerProgram
    );
    await program.methods
      .lockCollateral(new BN(amount))
      .accounts({
        vault: vaultPda(owner),
        vaultAuthority,
        callerProgram,
        cpiAuthority: authority,
      })
      .rpc();
  }

  async function unlockDirect(owner: PublicKey, amount: number, callerProgram: PublicKey) {
    const [authority] = PublicKey.findProgramAddressSync(
      [Buffer.from("cpi_authority")],
      callerProgram
    );
    await program.methods
      .unlockCollateral(new BN(amount))
      .accounts({
        vault: vaultPda(owner),
        vaultAuthority,
        callerProgram,
        cpiAuthority: authority,
      })
      .rpc();
  }

  async function transferDirect(
    from: PublicKey,
    to: PublicKey,
    amount: number,
    callerProgram: PublicKey
  ) {
    const [authority] = PublicKey.findProgramAddressSync(
      [Buffer.from("cpi_authority")],
      callerProgram
    );
    await program.methods
      .transferCollateral(new BN(amount))
      .accounts({
        fromVault: vaultPda(from),
        toVault: vaultPda(to),
        fromVaultTokenAccount: await vaultToken(from),
        toVaultTokenAccount: await vaultToken(to),
        mint,
        toVaultAuthority: vaultPda(to),
        fromVaultAuthority: vaultPda(from),
        vaultAuthority,
        callerProgram,
        cpiAuthority: authority,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();
  }

  before(async () => {
    await airdrop(userA.publicKey);
    await airdrop(userB.publicKey);
    await airdrop(userC.publicKey);
    await airdrop(userD.publicKey);
    await airdrop(admin.publicKey);

    mint = await createMint(provider.connection, admin, admin.publicKey, null, 6);

    [vaultAuthority] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault_authority")],
      program.programId
    );

    try {
      await program.account.vaultAuthority.fetch(vaultAuthority);
    } catch {
      await program.methods
        .initializeVaultAuthority([testCaller.programId])
        .accounts({
          upgradeAuthority: admin.publicKey,
          program: program.programId,
          programData: programDataPda(program.programId),
          vaultAuthority,
          systemProgram: SystemProgram.programId,
        })
        .rpc();
    }

    for (const user of [userA, userB, userC, userD]) {
      const ata = await createAssociatedTokenAccount(
        provider.connection,
        user,
        mint,
        user.publicKey
      );
      await mintTo(provider.connection, admin, mint, ata, admin, 20_000_000);
      await initVault(user);
    }

    await deposit(userA, 10_000_000);
    await deposit(userB, 1_000_000);
    await assertVaultTokens(userA.publicKey);
    await assertVaultTokens(userB.publicKey);
  });

  it("locks collateral through authorized cpi", async () => {
    await lockCpi(userA.publicKey, 100_000);
    const vault = await fetchVault(userA.publicKey);
    assert.equal(vault.lockedBalance.toNumber(), 100_000);
    assert.equal(vault.availableBalance.toNumber(), 9_900_000);
    await assertVaultTokens(userA.publicKey);
  });

  it("unlocks collateral through authorized cpi", async () => {
    await unlockCpi(userA.publicKey, 100_000);
    const vault = await fetchVault(userA.publicKey);
    assert.equal(vault.lockedBalance.toNumber(), 0);
    assert.equal(vault.availableBalance.toNumber(), 10_000_000);
    await assertVaultTokens(userA.publicKey);
  });

  it("transfers collateral through authorized cpi", async () => {
    await transferCpi(userA.publicKey, userB.publicKey, 50_000);
    const fromVault = await fetchVault(userA.publicKey);
    const toVault = await fetchVault(userB.publicKey);
    assert.equal(fromVault.totalBalance.toNumber(), 9_950_000);
    assert.equal(fromVault.availableBalance.toNumber(), 9_950_000);
    assert.equal(toVault.totalBalance.toNumber(), 1_050_000);
    await assertVaultTokens(userA.publicKey);
    await assertVaultTokens(userB.publicKey);
  });

  it("rejects lock from a direct caller", async () => {
    await expectFail(() => lockDirect(userA.publicKey, 1, TOKEN_PROGRAM_ID));
  });

  it("rejects unlock from a direct caller", async () => {
    await expectFail(() => unlockDirect(userA.publicKey, 1, TOKEN_PROGRAM_ID));
  });

  it("rejects transfer from a direct caller", async () => {
    await expectFail(() =>
      transferDirect(userA.publicKey, userB.publicKey, 1, TOKEN_PROGRAM_ID)
    );
  });

  it("rejects lock that only passes an authorized program id", async () => {
    await expectFail(() => lockDirect(userA.publicKey, 1, testCaller.programId));
  });

  it("rejects lock above available balance", async () => {
    const vault = await fetchVault(userA.publicKey);
    await expectFail(() =>
      lockCpi(userA.publicKey, vault.availableBalance.toNumber() + 1)
    );
  });

  it("rejects unlock above locked balance", async () => {
    await lockCpi(userA.publicKey, 100);
    await expectFail(() => unlockCpi(userA.publicKey, 101));
    await unlockCpi(userA.publicKey, 100);
  });

  it("rejects withdraw of locked collateral", async () => {
    const before = await fetchVault(userA.publicKey);
    await lockCpi(userA.publicKey, before.availableBalance.toNumber());
    await expectFail(() => withdraw(userA, 1));
    await unlockCpi(userA.publicKey, before.availableBalance.toNumber());
  });

  it("rejects zero amounts", async () => {
    await expectFail(() => deposit(userA, 0));
    await expectFail(() => withdraw(userA, 0));
    await expectFail(() => lockCpi(userA.publicKey, 0));
    await expectFail(() => unlockCpi(userA.publicKey, 0));
    await expectFail(() => transferCpi(userA.publicKey, userB.publicKey, 0));
  });

  it("keeps total equal to available plus locked", async () => {
    await deposit(userC, 1_000_000);
    await assertVaultTokens(userC.publicKey);

    await lockCpi(userC.publicKey, 400_000);
    let vault = await fetchVault(userC.publicKey);
    assert.equal(vault.totalBalance.toNumber(), 1_000_000);
    assert.equal(vault.availableBalance.toNumber(), 600_000);
    assert.equal(vault.lockedBalance.toNumber(), 400_000);
    await assertVaultTokens(userC.publicKey);

    await unlockCpi(userC.publicKey, 100_000);
    vault = await fetchVault(userC.publicKey);
    assert.equal(vault.availableBalance.toNumber(), 700_000);
    assert.equal(vault.lockedBalance.toNumber(), 300_000);
    await assertVaultTokens(userC.publicKey);

    await transferCpi(userC.publicKey, userB.publicKey, 200_000);
    vault = await fetchVault(userC.publicKey);
    assert.equal(vault.totalBalance.toNumber(), 800_000);
    assert.equal(vault.availableBalance.toNumber(), 500_000);
    assert.equal(vault.lockedBalance.toNumber(), 300_000);
    await assertVaultTokens(userC.publicKey);
    await assertVaultTokens(userB.publicKey);

    await withdraw(userC, 100_000);
    vault = await fetchVault(userC.publicKey);
    assert.equal(vault.totalBalance.toNumber(), 700_000);
    assert.equal(vault.availableBalance.toNumber(), 400_000);
    assert.equal(vault.lockedBalance.toNumber(), 300_000);
    await assertVaultTokens(userC.publicKey);
    await assertVaultTokens(userB.publicKey);
  });

  it("does not credit unsolicited token transfers", async () => {
    await deposit(userD, 500_000);
    const before = await fetchVault(userD.publicKey);
    const vaultAta = await vaultToken(userD.publicKey);
    const beforeTokens = await tokenAmount(vaultAta);
    const surplus = 100_000;

    await transfer(
      provider.connection,
      userD,
      await userToken(userD.publicKey),
      vaultAta,
      userD,
      surplus
    );

    const after = await fetchVault(userD.publicKey);
    const afterTokens = await tokenAmount(vaultAta);

    assert.equal(after.totalBalance.toNumber(), before.totalBalance.toNumber());
    assert.equal(
      after.availableBalance.toNumber(),
      before.availableBalance.toNumber()
    );
    assert.equal(afterTokens, beforeTokens + surplus);
    assert.isAbove(afterTokens, after.totalBalance.toNumber());
    await expectFail(() =>
      withdraw(userD, after.availableBalance.toNumber() + 1)
    );
  });
});
