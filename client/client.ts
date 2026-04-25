import * as anchor from "@coral-xyz/anchor";
import BN from "bn.js";
import * as web3 from "@solana/web3.js";
import type { ScoreTracker } from "../target/types/score_tracker";

// Configure the client to use the local cluster
anchor.setProvider(anchor.AnchorProvider.env());

const program = anchor.workspace.ScoreTracker as anchor.Program<ScoreTracker>;

// Client
// Derivamos el PDA del perfil
const [profilePda] = web3.PublicKey.findProgramAddressSync(
  [Buffer.from("profile"), program.provider.publicKey.toBuffer()],
  program.programId
);

console.log("Wallet:", program.provider.publicKey.toBase58());
console.log("PDA del jugador:", profilePda.toBase58());

// -----------------------------------------------------------------------------
// CREATE
// -----------------------------------------------------------------------------
const txCreate = await program.methods
  .createPlayer("eber_dev")
  .accounts({
    profile: profilePda,
    authority: program.provider.publicKey,
  })
  .rpc();
console.log("create_player tx:", txCreate);

// -----------------------------------------------------------------------------
// READ
// -----------------------------------------------------------------------------
let profile = await program.account.playerProfile.fetch(profilePda);
console.log("Perfil inicial:", {
  username: profile.username,
  score: profile.score.toString(),
  gamesPlayed: profile.gamesPlayed.toString(),
});

// -----------------------------------------------------------------------------
// UPDATE - sumar puntos en dos partidas
// -----------------------------------------------------------------------------
await program.methods
  .submitScore(new BN(150))
  .accounts({ profile: profilePda, authority: program.provider.publicKey })
  .rpc();

await program.methods
  .submitScore(new BN(80))
  .accounts({ profile: profilePda, authority: program.provider.publicKey })
  .rpc();

profile = await program.account.playerProfile.fetch(profilePda);
console.log("Despues de jugar:", {
  score: profile.score.toString(),
  gamesPlayed: profile.gamesPlayed.toString(),
});

// -----------------------------------------------------------------------------
// UPDATE - cambiar username
// -----------------------------------------------------------------------------
await program.methods
  .updateUsername("eber_pro")
  .accounts({ profile: profilePda, authority: program.provider.publicKey })
  .rpc();

profile = await program.account.playerProfile.fetch(profilePda);
console.log("Username actualizado:", profile.username);

// -----------------------------------------------------------------------------
// DELETE - cerrar el perfil y recuperar el rent
// -----------------------------------------------------------------------------
await program.methods
  .deletePlayer()
  .accounts({ profile: profilePda, authority: program.provider.publicKey })
  .rpc();

try {
  await program.account.playerProfile.fetch(profilePda);
  console.log("ERROR: la cuenta deberia estar cerrada");
} catch (e) {
  console.log("Perfil eliminado correctamente.");
}