import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { AnchorAmm } from "../target/types/anchor_amm";
import {
  PublicKey,
  Keypair,
  SystemProgram,
} from "@solana/web3.js";
import { BN } from "@coral-xyz/anchor";
import { assert } from "chai";

import {
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  createMint,
  createAssociatedTokenAccount,
  mintTo,
  getAssociatedTokenAddress,
} from "@solana/spl-token";

describe("anchor-amm", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.AnchorAmm as Program<AnchorAmm>;
  const wallet = provider.wallet as anchor.Wallet;

  let mintX: PublicKey;
  let mintY: PublicKey;
  let mintLP: PublicKey;
  let userX: PublicKey;
  let userY: PublicKey;
  let userLP: PublicKey;
  let vaultX: PublicKey;
  let vaultY: PublicKey;
  let config: PublicKey;
  
  const seed = new BN(Math.floor(Math.random() * 1000000));
  const fee = 30;
  
  const deriveLPMint = async (config: PublicKey) => {
    return PublicKey.findProgramAddressSync(
      [Buffer.from("lp"), config.toBuffer()],
      program.programId
    );
  };

  const deriveConfig = async (seed: anchor.BN) => {
    return PublicKey.findProgramAddressSync(
      [Buffer.from("config"), seed.toArrayLike(Buffer, "le", 8)],
      program.programId
    );
  };

  before(async function() {
    this.timeout(60000);
    
    [config] = await deriveConfig(seed);
    [mintLP] = await deriveLPMint(config);
    
    mintX = await createMint(
      provider.connection,
      wallet.payer,
      wallet.publicKey,
      null,
      6
    );
    
    mintY = await createMint(
      provider.connection,
      wallet.payer,
      wallet.publicKey,
      null,
      6
    );
    
    userX = await createAssociatedTokenAccount(
      provider.connection,
      wallet.payer,
      mintX,
      wallet.publicKey
    );
    
    userY = await createAssociatedTokenAccount(
      provider.connection,
      wallet.payer,
      mintY,
      wallet.publicKey
    );
    
    await mintTo(
      provider.connection,
      wallet.payer,
      mintX,
      userX,
      wallet.publicKey,
      1_000_000_000
    );
    
    await mintTo(
      provider.connection,
      wallet.payer,
      mintY,
      userY,
      wallet.publicKey,
      1_000_000_000
    );
    
    vaultX = await getAssociatedTokenAddress(
      mintX,
      config,
      true
    );
    
    vaultY = await getAssociatedTokenAddress(
      mintY,
      config,
      true
    );
  });

  it("Initialize AMM pool", async () => {
    const tx = await program.methods
      .initialize(seed, fee)
      .accountsStrict({
        initializer: wallet.publicKey,
        mintX,
        mintY,
        mintLp: mintLP,
        vaultX,
        vaultY,
        config,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
    
    const configAccount = await program.account.config.fetch(config);
    assert.equal(configAccount.seed.toString(), seed.toString());
    assert.equal(configAccount.mintX.toString(), mintX.toString());
    assert.equal(configAccount.mintY.toString(), mintY.toString());
    assert.equal(configAccount.fee, fee);
    assert.equal(configAccount.locked, false);
    
    const vaultXInfo = await provider.connection.getTokenAccountBalance(vaultX);
    const vaultYInfo = await provider.connection.getTokenAccountBalance(vaultY);
    assert.equal(vaultXInfo.value.amount, "0");
    assert.equal(vaultYInfo.value.amount, "0");
  });

  it("Deposit into the AMM pool", async () => {
    userLP = await createAssociatedTokenAccount(
      provider.connection,
      wallet.payer,
      mintLP,
      wallet.publicKey
    );
    
    const amount = new BN(50_000_000);
    const minX = new BN(0);
    const minY = new BN(0);
    
    const userXBalanceBefore = await provider.connection.getTokenAccountBalance(userX);
    const userYBalanceBefore = await provider.connection.getTokenAccountBalance(userY);
    const vaultXBalanceBefore = await provider.connection.getTokenAccountBalance(vaultX);
    const vaultYBalanceBefore = await provider.connection.getTokenAccountBalance(vaultY);
    
    const tx = await program.methods
      .addLiquidity(amount, minX, minY)
      .accountsStrict({
        user: wallet.publicKey,
        mintX,
        mintY,
        mintLp: mintLP,
        vaultX,
        vaultY,
        userX,
        userY,
        userLp: userLP,
        config,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
    
    const userXBalanceAfter = await provider.connection.getTokenAccountBalance(userX);
    const userYBalanceAfter = await provider.connection.getTokenAccountBalance(userY);
    const vaultXBalanceAfter = await provider.connection.getTokenAccountBalance(vaultX);
    const vaultYBalanceAfter = await provider.connection.getTokenAccountBalance(vaultY);
    const userLpBalanceAfter = await provider.connection.getTokenAccountBalance(userLP);
    
    assert.equal(
      new BN(userXBalanceBefore.value.amount).sub(new BN(userXBalanceAfter.value.amount)).toString(),
      amount.toString()
    );
    assert.equal(
      new BN(userYBalanceBefore.value.amount).sub(new BN(userYBalanceAfter.value.amount)).toString(),
      amount.toString()
    );
    
    assert.equal(
      new BN(vaultXBalanceAfter.value.amount).sub(new BN(vaultXBalanceBefore.value.amount)).toString(),
      amount.toString()
    );
    assert.equal(
      new BN(vaultYBalanceAfter.value.amount).sub(new BN(vaultYBalanceBefore.value.amount)).toString(),
      amount.toString()
    );
    
    if (new BN(vaultXBalanceBefore.value.amount).eq(new BN(0)) &&
        new BN(vaultYBalanceBefore.value.amount).eq(new BN(0))) {
      const expectedLpAmount = Math.floor(Math.sqrt(
        Number(amount.toString()) * Number(amount.toString())
      ));
      
      assert.approximately(
        parseInt(userLpBalanceAfter.value.amount), 
        expectedLpAmount, 
        10,
        "LP token amount should match sqrt(amountX * amountY)"
      );
    } else {
      assert.isAbove(
        parseInt(userLpBalanceAfter.value.amount),
        0,
        "User should have received LP tokens"
      );
    }
  });

  it("Swap tokens in the AMM pool", async () => {
    const amountIn = new BN(10_000_000);
    const minAmountOut = new BN(1);
    
    const userXBalanceBefore = await provider.connection.getTokenAccountBalance(userX);
    const userYBalanceBefore = await provider.connection.getTokenAccountBalance(userY);
    const vaultXBalanceBefore = await provider.connection.getTokenAccountBalance(vaultX);
    const vaultYBalanceBefore = await provider.connection.getTokenAccountBalance(vaultY);
    
    const feeNumerator = fee;
    const feeDenominator = 10000;
    const amountInAfterFee = amountIn.toNumber() * (feeDenominator - feeNumerator) / feeDenominator;
    
    const vaultXAmount = parseInt(vaultXBalanceBefore.value.amount);
    const vaultYAmount = parseInt(vaultYBalanceBefore.value.amount);
    
    const expectedAmountOut = Math.floor(
      vaultYAmount - 
      (vaultXAmount * vaultYAmount) / 
      (vaultXAmount + amountInAfterFee)
    );
    
    const tx = await program.methods
      .swap(amountIn, minAmountOut, true)
      .accountsStrict({
        user: wallet.publicKey,
        mintX,
        mintY,
        vaultX,
        vaultY,
        userX,
        userY,
        config,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
    
    const userXBalanceAfter = await provider.connection.getTokenAccountBalance(userX);
    const userYBalanceAfter = await provider.connection.getTokenAccountBalance(userY);
    const vaultXBalanceAfter = await provider.connection.getTokenAccountBalance(vaultX);
    const vaultYBalanceAfter = await provider.connection.getTokenAccountBalance(vaultY);
    
    assert.equal(
      new BN(userXBalanceBefore.value.amount).sub(new BN(userXBalanceAfter.value.amount)).toString(),
      amountIn.toString()
    );
    
    const actualAmountOut = new BN(userYBalanceAfter.value.amount).sub(new BN(userYBalanceBefore.value.amount));
    
    assert.approximately(
      actualAmountOut.toNumber(),
      expectedAmountOut,
      10,
      "Output amount should match the expected amount based on the constant product formula"
    );
    
    assert.equal(
      new BN(vaultXBalanceAfter.value.amount).sub(new BN(vaultXBalanceBefore.value.amount)).toString(),
      amountIn.toString()
    );
    
    assert.equal(
      new BN(vaultYBalanceBefore.value.amount).sub(new BN(vaultYBalanceAfter.value.amount)).toString(),
      actualAmountOut.toString()
    );
    
    const productBefore = new BN(vaultXBalanceBefore.value.amount).mul(new BN(vaultYBalanceBefore.value.amount));
    const productAfter = new BN(vaultXBalanceAfter.value.amount).mul(new BN(vaultYBalanceAfter.value.amount));
    
    assert.isAtLeast(parseInt(productAfter.toString()), parseInt(productBefore.toString()), "Constant product invariant should hold");
    
    const reverseAmountIn = new BN(5_000_000);
    
    const reverseTx = await program.methods
      .swap(reverseAmountIn, minAmountOut, false)
      .accountsStrict({
        user: wallet.publicKey,
        mintX,
        mintY,
        vaultX,
        vaultY,
        userX,
        userY,
        config,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
  });
});
