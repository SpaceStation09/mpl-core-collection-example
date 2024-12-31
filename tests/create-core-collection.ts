import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { CreateCoreCollection } from "../target/types/create_core_collection";
import { createUmi } from "@metaplex-foundation/umi-bundle-defaults";
import {
  fetchAsset,
  fetchCollection,
  Key,
} from "@metaplex-foundation/mpl-core";
import { expect } from "chai";

describe("create-core-collection", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  // const wallet = anchor.Wallet.local();

  const program = anchor.workspace
    .CreateCoreCollection as Program<CreateCoreCollection>;
  const umi = createUmi(provider.connection);
  const collection = anchor.web3.Keypair.generate();

  it("Can create collection!", async () => {
    // Add your test here.
    await program.methods
      .createCollection({
        name: "My Collection",
        uri: "https://example.com",
      })
      .accounts({
        collection: collection.publicKey,
        payer: anchor.getProvider().publicKey,
        updateAuthority: null,
      })
      .signers([collection])
      .rpc();
    // console.log("Your transaction signature", tx);

    const collectionAsset = await fetchCollection(
      umi,
      collection.publicKey.toBase58()
    );

    expect(collectionAsset.name).to.eq("My Collection");
  });

  it("Can create an asset", async () => {
    const asset = anchor.web3.Keypair.generate();
    await program.methods
      .createAsset({ name: "My asset", uri: "https://asset.example.com" })
      .accounts({
        asset: asset.publicKey,
        collection: collection.publicKey,
        authority: null,
        payer: anchor.getProvider().publicKey,
        owner: null,
        updateAuthority: null,
      })
      .signers([asset])
      .rpc();
    const assetInfo = await fetchAsset(umi, asset.publicKey.toBase58());
  });

  // it("Cannot transfer", async () => {
  //   const asset = anchor.web3.Keypair.generate();
  //   await program.methods
  //     .createAsset({ name: "My asset 2", uri: "https://asset2.example.com" })
  //     .accounts({
  //       asset: asset.publicKey,
  //       collection: collection.publicKey,
  //       authority: null,
  //       payer: anchor.getProvider().publicKey,
  //       owner: null,
  //       updateAuthority: null,
  //     })
  //     .signers([asset])
  //     .rpc();

  // });
});
