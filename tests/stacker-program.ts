import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { StackerProgram } from "../target/types/stacker_program";
import fs from "node:fs";
import { getOrCreateAssociatedTokenAccount } from "@solana/spl-token";
import umi from "../../utils/umi";
import { publicKey } from "@metaplex-foundation/umi";
import { expect } from "chai";
import { BN } from "bn.js";
import { fetchAssetV1 } from "@metaplex-foundation/mpl-core";

// TODO: Add tests for UserWalletState change

const walletSeeds = JSON.parse(
  fs.readFileSync(process.env.HOME + "/.config/solana/id.json", "utf8")
);

const keys = JSON.parse(fs.readFileSync("../keys.json", "utf8"));
const nfts = JSON.parse(fs.readFileSync("../nfts.json", "utf8"));
const keys_uns = JSON.parse(fs.readFileSync("../keys_uns.json", "utf8"));

const supportedMint = anchor.web3.Keypair.fromSecretKey(
  new Uint8Array(keys.mint.secret)
);
const unsupportedMint = anchor.web3.Keypair.fromSecretKey(
  new Uint8Array(keys_uns.mint.secret)
);

const supportedCollection = anchor.web3.Keypair.fromSecretKey(
  new Uint8Array(keys.collection.secret)
);
const unSupportedCollection = anchor.web3.Keypair.fromSecretKey(
  new Uint8Array(keys_uns.collection.secret)
);

const unSupportedCollectionNft = new anchor.web3.PublicKey(
  "6j17eddGieLbKXhVTqBcYLyD5tVyCcnhyANnSJKNM7Lq"
);

describe("stacker-program", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const reward_wallet = anchor.web3.Keypair.generate();

  const signer = anchor.web3.Keypair.fromSecretKey(new Uint8Array(walletSeeds));
  const token_seed_index_bytes = Buffer.alloc(4);
  token_seed_index_bytes.writeUInt32BE(0);
  // const unlockPeriodLong = 1 * 24 * 60 * 60; // in seconds
  const unlockPeriodShort = 7; // in seconds

  const id = "0";
  // const mintAmount = BigInt("1000000000000000");
  const stackAmount = "100000000000";
  const early_unstacking_tax = 90;
  const program = anchor.workspace.StackerProgram as Program<StackerProgram>;

  const [stackMetadataPda] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("stacker"), Buffer.from("metadata")],
    program.programId
  );

  const [userStackPda] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("stacker"), signer.publicKey.toBuffer()],
    program.programId
  );

  const [stack_asset_metadata_token] =
    anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("stacker"),
        Buffer.from("token"),
        signer.publicKey.toBuffer(),
        Buffer.from(id),
      ],
      program.programId
    );

  const [stack_asset_metadata_nft] =
    anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("stacker"),
        Buffer.from("nft"),
        signer.publicKey.toBuffer(),
        Buffer.from(id),
      ],
      program.programId
    );

  it("should not initialize stacker account if force_unstack_tax is invalid", async () => {
    try {
      await program.methods
        .initialize(110)
        .accounts({
          mint: supportedMint.publicKey,
          rewardWallet: reward_wallet.publicKey,
          collection: supportedCollection.publicKey,
        })
        .rpc();
    } catch (err) {
      expect(err.error?.errorCode?.code).to.eq("ForceUnlockTaxInvalid");
    }
  });

  it("should initialize stacker account", async () => {
    await getOrCreateAssociatedTokenAccount(
      provider.connection,
      signer,
      supportedMint.publicKey,
      reward_wallet.publicKey
    );
    // Try to initialize wallet
    await program.methods
      .initialize(early_unstacking_tax)
      .accounts({
        mint: supportedMint.publicKey,
        rewardWallet: reward_wallet.publicKey,
        collection: supportedCollection.publicKey,
      })
      .rpc();

    // Fetch Account Data and verify its correctness
    const stackerMetadata = await program.account.stackerMetadata.fetch(
      stackMetadataPda
    );
    expect(stackerMetadata).to.deep.eq({
      decimals: 7,
      mint: supportedMint.publicKey,
      collectionMint: supportedCollection.publicKey,
      owner: signer.publicKey,
      isDisabled: false,
      rewardWallet: reward_wallet.publicKey,
      forceUnlockTax: early_unstacking_tax,
    });
  });

  it("should create user stack account", async () => {
    // Initialize User ATA also, but only for testing
    // IRL user should have already initialized ATA as
    // user must own our tokens before interacting with this stacker program
    await getOrCreateAssociatedTokenAccount(
      provider.connection,
      signer,
      supportedMint.publicKey,
      signer.publicKey
    );
    await program.methods
      .createLockerAccount()
      .accounts({ mint: supportedMint.publicKey })
      .rpc();
    const userStackAccount = await program.account.userStackPda.fetch(
      userStackPda
    );
    expect(userStackAccount.stackedNfts).to.eq(0);
    expect(userStackAccount.stackedTokens.toString()).to.eq("0");
  });

  it("should not lock tokens if token amount is invalid", async () => {
    try {
      await program.methods
        .lockTokens(new BN(0), new BN(unlockPeriodShort), id)
        .accounts({ mint: supportedMint.publicKey })
        .rpc();
      throw new Error("Validation bypassed");
    } catch (err) {
      expect(err.error?.errorCode?.code).to.eq("InvalidAmount");
    }
  });

  it("should not lock tokens if token duration is invalid", async () => {
    try {
      await program.methods
        .lockTokens(new BN(stackAmount), new BN(0), id)
        .accounts({ mint: supportedMint.publicKey })
        .rpc();
      throw new Error("Validation bypassed");
    } catch (err) {
      expect(err.error?.errorCode?.code).to.eq("InvalidDuration");
    }
  });

  it("should not lock tokens if token is not supported for locking", async () => {
    // Initialize User ATA also, but only for testing.
    // IRL user may have already initialized ATA.
    // In case user does not has ATA, it will fail
    // With other error.
    await getOrCreateAssociatedTokenAccount(
      provider.connection,
      signer,
      unsupportedMint.publicKey,
      signer.publicKey
    );
    await getOrCreateAssociatedTokenAccount(
      provider.connection,
      signer,
      unsupportedMint.publicKey,
      userStackPda,
      true
    );
    try {
      await program.methods
        .lockTokens(new BN(stackAmount), new BN(unlockPeriodShort), id)
        .accounts({ mint: unsupportedMint.publicKey })
        .rpc();
      throw new Error("Validation bypassed");
    } catch (err) {
      expect(err.error?.errorCode?.code).to.eq("UnsupportedTokenMint");
    }
  });

  it("should not try to lock tokens if passed token amount is more then user owns", async () => {
    try {
      const userAta = await getOrCreateAssociatedTokenAccount(
        provider.connection,
        signer,
        supportedMint.publicKey,
        signer.publicKey
      );
      await program.methods
        .lockTokens(
          new BN((userAta.amount + BigInt(5)).toString()),
          new BN(unlockPeriodShort),
          id
        )
        .accounts({ mint: supportedMint.publicKey })
        .rpc();
      throw new Error("Validation bypassed");
    } catch (err) {
      expect(err.error?.errorCode?.code).to.eq("InsufficientTokenBalance");
    }
  });

  it("should lock token assets", async () => {
    let isEventValid: boolean | null = null;
    const sid = program.addEventListener("lockAssetEvent", (e) => {
      if (e.lockedBy.toString() === signer.publicKey.toString())
        isEventValid = true;
      else {
        isEventValid = false;
        console.error("Inavlid Event Data found!", e);
      }
      program.removeEventListener(sid);
    });
    await program.methods
      .lockTokens(new BN(stackAmount), new BN(unlockPeriodShort), id)
      .accounts({ mint: supportedMint.publicKey })
      .rpc();

    const userEscrowAccount = await program.account.userStackPda.fetch(
      userStackPda
    );
    const lock_asset_metadata = await program.account.stackAssetMetadata.fetch(
      stack_asset_metadata_token
    );

    const userStackPdaAta = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      signer,
      supportedMint.publicKey,
      userStackPda,
      true
    );

    expect(userEscrowAccount.stackedTokens.toString()).to.eq(stackAmount);

    expect(lock_asset_metadata.assetKeys[0].toString()).to.eq(
      supportedMint.publicKey.toString()
    );
    expect(lock_asset_metadata.duration.toString()).to.eq(
      unlockPeriodShort.toString()
    );
    expect(lock_asset_metadata.isCleared).to.be.false;
    expect(lock_asset_metadata.isNft).to.be.false;
    expect(lock_asset_metadata.remainingAmount.toString()).to.eq(stackAmount);
    expect(lock_asset_metadata.totalAmount.toString()).to.eq(stackAmount);

    expect(userStackPdaAta.amount.toString()).to.eq(stackAmount);
    const start = Date.now();
    while (true) {
      if (Date.now() > start + 20000 || isEventValid !== null) break;
    }
    program.removeEventListener(sid);
    expect(isEventValid).to.eq(true);
  });

  it("should not unlock tokens if locking period is not elipsed", async () => {
    try {
      await program.methods
        .unlockTokens(id, false)
        .accounts({
          mint: supportedMint.publicKey,
          rewardAccount: reward_wallet.publicKey,
        })
        .rpc();
      throw new Error("Validation bypassed");
    } catch (err) {
      expect(err.error?.errorCode?.code).to.eq("LockDurationNotCompleted");
    }
  });

  it("should not try to lock nft if nft is not owned by signer", async () => {
    try {
      await program.methods
        .lockNft(new BN(unlockPeriodShort), id)
        .accounts({
          collection: supportedCollection.publicKey,
        })
        .remainingAccounts(
          nfts.map((n: any) => ({
            isSigner: false,
            isWritable: true,
            pubkey: new anchor.web3.PublicKey(n.pub),
          }))
        )
        .rpc();
      throw new Error("Validation bypassed");
    } catch (err) {
      expect(err.error?.errorCode?.code).to.eq("NotNFTOwner");
    }
  });

  it("should not lock nft asset if nft collection is not supported for locking", async () => {
    try {
      await program.methods
        .lockNft(new BN(unlockPeriodShort), id)
        .accounts({
          collection: unSupportedCollection.publicKey,
        })
        .remainingAccounts([
          {
            isSigner: false,
            isWritable: true,
            pubkey: unSupportedCollectionNft,
          },
        ])
        .rpc();
      throw new Error("Validation bypassed");
    } catch (err) {
      expect(err.error?.errorCode?.code).to.eq("UnsupportedCollection");
    }
  });

  it("should lock nft asset", async () => {
    let isEventValid: boolean | null = null;
    const sid = program.addEventListener("lockAssetEvent", (e) => {
      for (let i = 0; i < e.nftKeys.length; i++) {
        const k = e.nftKeys[i];
        if (k.toString() !== nfts[i + 1].pub) {
          isEventValid = false;
          console.error("Invalid Event Data Received: ", e);
          break;
        }
      }
      if (isEventValid === null) isEventValid = true;
      program.removeEventListener(sid);
    });
    const tx = await program.methods
      .lockNft(new BN(13), id)
      .accounts({
        collection: supportedCollection.publicKey,
      })
      .remainingAccounts(
        nfts.slice(1).map((n: any) => ({
          isSigner: false,
          isWritable: true,
          pubkey: new anchor.web3.PublicKey(n.pub),
        }))
      )
      .rpc();

    await provider.connection.confirmTransaction(
      {
        signature: tx,
      } as anchor.web3.TransactionConfirmationStrategy,
      "finalized"
    );

    const assetsPromise = nfts.slice(1).map((n: any) => {
      return fetchAssetV1(umi, publicKey(n.pub));
    });

    const assets = await Promise.all(assetsPromise);

    for (let asset of assets) {
      expect(asset.owner).to.eq(userStackPda.toString());
    }

    const lock_asset_metadata = await program.account.stackAssetMetadata.fetch(
      stack_asset_metadata_nft
    );
    const userEscrowAccount = await program.account.userStackPda.fetch(
      userStackPda
    );

    expect(userEscrowAccount.stackedNfts).to.eq(nfts.length - 1);

    for (let i = 0; i < lock_asset_metadata.assetKeys.length; i++) {
      expect(lock_asset_metadata.assetKeys[i].toString()).to.eq(
        nfts.slice(1)[i].pub
      );
    }

    expect(lock_asset_metadata.duration.toString()).to.eq("13");
    expect(lock_asset_metadata.isCleared).to.eq(false);
    expect(lock_asset_metadata.isNft).to.eq(true);
    expect(lock_asset_metadata.remainingAmount.toString()).to.eq(
      `${nfts.length - 1}`
    );
    expect(lock_asset_metadata.totalAmount.toString()).to.eq(
      `${nfts.length - 1}`
    );
    const start = Date.now();
    while (true) {
      if (Date.now() > start + 20000 || isEventValid !== null) break;
    }
    program.removeEventListener(sid);
    expect(isEventValid).to.eq(true);
  });

  it("should not unlock nft assets before locking time elipsed", async () => {
    try {
      await program.methods
        .unlockNfts(id)
        .accounts({
          collection: supportedCollection.publicKey,
        })
        .remainingAccounts(
          nfts.slice(1).map((n: any) => ({
            isSigner: false,
            isWritable: true,
            pubkey: new anchor.web3.PublicKey(n.pub),
          }))
        )
        .rpc();
    } catch (err) {
      expect(err.error?.errorCode?.code).to.eq("LockDurationNotCompleted");
    }
  });

  it("should unlock token asset", async () => {
    let isEventValid: boolean | null = null;
    const sid = program.addEventListener("unlockAssetsEvent", (e) => {
      if (
        e.lockAssetMetadata.toString() === stack_asset_metadata_token.toString()
      )
        isEventValid = true;
      else {
        isEventValid = false;
        console.error("Inavlid Event Data found!", e);
      }
      program.removeEventListener(sid);
    });
    const userAtaBefore = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      signer,
      supportedMint.publicKey,
      signer.publicKey
    );

    await program.methods
      .unlockTokens(id, false)
      .accounts({
        mint: supportedMint.publicKey,
        rewardAccount: reward_wallet.publicKey,
      })
      .rpc();

    const lock_asset_metadata = await program.account.stackAssetMetadata.fetch(
      stack_asset_metadata_token
    );
    const userAtaAfter = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      signer,
      supportedMint.publicKey,
      signer.publicKey
    );
    const userEscrowAccount = await program.account.userStackPda.fetch(
      userStackPda
    );
    const userStackPdaAta = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      signer,
      supportedMint.publicKey,
      userStackPda,
      true
    );
    expect(userAtaBefore.amount + BigInt(stackAmount)).to.eq(
      userAtaAfter.amount
    );
    expect(userEscrowAccount.stackedTokens.toString()).to.eq("0");
    expect(userStackPdaAta.amount.toString()).to.eq("0");
    expect(lock_asset_metadata.remainingAmount.toString()).to.eq("0");
    expect(lock_asset_metadata.isCleared).to.eq(true);
    const start = Date.now();
    while (true) {
      if (Date.now() > start + 20000 || isEventValid !== null) break;
    }
    program.removeEventListener(sid);
    expect(isEventValid).to.eq(true);
  });

  it("should not unlock tokens if tokens are already released", async () => {
    try {
      await program.methods
        .unlockTokens(id, false)
        .accounts({
          mint: supportedMint.publicKey,
          rewardAccount: reward_wallet.publicKey,
        })
        .rpc();
      throw new Error("Validation bypassed");
    } catch (err) {
      expect(err.error?.errorCode?.code).to.eq("AlreadyReleased");
    }
  });

  it("should not try to unlock nft assets if collection is unsupported", async () => {
    try {
      console.log("Waiting for NFT asset to unlock...");
      await new Promise((res) => setTimeout(res, 8000));
      await program.methods
        .unlockNfts(id)
        .accounts({
          collection: unSupportedCollection.publicKey,
        })
        .remainingAccounts(
          nfts.slice(1).map((n: any) => ({
            isSigner: false,
            isWritable: true,
            pubkey: new anchor.web3.PublicKey(n.pub),
          }))
        )
        .rpc();
    } catch (err) {
      expect(err.error?.errorCode?.code).to.eq("UnsupportedCollection");
    }
  });

  it("should not try to unlock nft assets if entered nft is invalid", async () => {
    try {
      const accs = nfts.slice(1).map((n: any) => ({
        isSigner: false,
        isWritable: true,
        pubkey: new anchor.web3.PublicKey(n.pub),
      }));
      accs[0].pubkey = unSupportedCollectionNft;

      await program.methods
        .unlockNfts(id)
        .accounts({
          collection: supportedCollection.publicKey,
        })
        .remainingAccounts(accs)
        .rpc();
    } catch (err) {
      expect(err.error?.errorCode?.code).to.eq("InvalidNFTAccountsPassed");
    }
  });

  it("should unlock nft assets", async () => {
    let isEventValid: boolean | null = null;
    const sid = program.addEventListener("unlockAssetsEvent", (e) => {
      for (let i = 0; i < e.nftKeys.length; i++) {
        const k = e.nftKeys[i];
        if (k.toString() !== nfts[i + 1].pub) {
          isEventValid = false;
          console.error("Invalid Event Data Received: ", e);
          break;
        }
      }
      if (isEventValid === null) isEventValid = true;
      program.removeEventListener(sid);
    });
    const tx = await program.methods
      .unlockNfts(id)
      .accounts({
        collection: supportedCollection.publicKey,
      })
      .remainingAccounts(
        nfts.slice(1).map((n: any) => ({
          isSigner: false,
          isWritable: true,
          pubkey: new anchor.web3.PublicKey(n.pub),
        }))
      )
      .rpc();

    await provider.connection.confirmTransaction(
      {
        signature: tx,
      } as anchor.web3.TransactionConfirmationStrategy,
      "finalized"
    );
    const assetsPromise = nfts.slice(1).map((n: any) => {
      return fetchAssetV1(umi, publicKey(n.pub));
    });
    const assets = await Promise.all(assetsPromise);

    for (let asset of assets) {
      expect(asset.owner).to.eq(signer.publicKey.toString());
    }

    const lock_asset_metadata = await program.account.stackAssetMetadata.fetch(
      stack_asset_metadata_nft
    );
    const userEscrowAccount = await program.account.userStackPda.fetch(
      userStackPda
    );

    expect(userEscrowAccount.stackedNfts).to.eq(0);
    expect(lock_asset_metadata.isCleared).to.eq(true);
    expect(lock_asset_metadata.remainingAmount.toString()).to.eq("0");
    const start = Date.now();
    while (true) {
      if (Date.now() > start + 20000 || isEventValid !== null) break;
    }
    program.removeEventListener(sid);
    expect(isEventValid).to.eq(true);
  });

  it("should not unlock nft assets if nfts are already released", async () => {
    try {
      await program.methods
        .unlockNfts(id)
        .accounts({
          collection: supportedCollection.publicKey,
        })
        .remainingAccounts(
          nfts.slice(1).map((n: any) => ({
            isSigner: false,
            isWritable: true,
            pubkey: new anchor.web3.PublicKey(n.pub),
          }))
        )
        .rpc();
    } catch (err) {
      expect(err.error?.errorCode?.code).to.eq("AlreadyReleased");
    }
  });

  it("should not disbale wallet if signer is not the initializer of wallet", async () => {
    try {
      const signer = anchor.web3.Keypair.generate();
      await program.methods
        .changeWalletState(true)
        .accounts({ owner: signer.publicKey })
        .signers([signer])
        .rpc();
    } catch (err) {
      expect(err.error?.errorCode?.code).to.eq("NotAllowed");
    }
  });

  it("should disable wallet", async () => {
    await program.methods.changeWalletState(true).rpc();
    const stackerMetadata = await program.account.stackerMetadata.fetch(
      stackMetadataPda
    );

    expect(stackerMetadata.isDisabled).to.eq(true);
  });

  it("should enable wallet", async () => {
    await program.methods.changeWalletState(false).rpc();
    const stackerMetadata = await program.account.stackerMetadata.fetch(
      stackMetadataPda
    );

    expect(stackerMetadata.isDisabled).to.eq(false);
  });

  // TODO: Complete Test implimentation
  it("sholud force unlock tokens with tax deduction", async () => {
    // Lock the tokens
    await program.methods
      .lockTokens(new BN(stackAmount), new BN(60 * 60), "1")
      .accounts({ mint: supportedMint.publicKey })
      .rpc();

    // Unlock immidiately with force set to 'true'
    await program.methods
      .unlockTokens("1", true)
      .accounts({
        mint: supportedMint.publicKey,
        rewardAccount: reward_wallet.publicKey,
      })
      .rpc();

    const userEscrowAccount = await program.account.userStackPda.fetch(
      userStackPda
    );

    const reward_wallet_ata = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      signer,
      supportedMint.publicKey,
      reward_wallet.publicKey
    );

    const taxAmount =
      (BigInt(stackAmount) * BigInt(early_unstacking_tax)) / BigInt(100);

    expect(reward_wallet_ata.amount.toString()).to.eq(taxAmount.toString());
  });
});
