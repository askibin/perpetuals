import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { Perpetuals } from "../target/types/perpetuals";

describe("perpetuals", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.Perpetuals as Program<Perpetuals>;

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  });
});
