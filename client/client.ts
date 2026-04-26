// client/client.ts — Hippocrates · CRUD demo en Devnet (Solana Playground)
//
// Solana Playground inyecta automáticamente:
//   - pg.wallet, pg.connection, pg.PROGRAM_ID, pg.program
// Este script ejecuta el flujo CRUD completo y deja trazas legibles en el log.

import { PublicKey, SystemProgram } from "@solana/web3.js";

const ID_CEDULA = "9876543";

// Hash demo (en producción viene de scripts/sep_query.ts)
const FAKE_HASH_PAYLOAD     = Array(32).fill(0).map((_, i) => (i * 7) % 256);
const FAKE_HASH_NOMBRE      = Array(32).fill(0).map((_, i) => (i * 11) % 256);
const FAKE_HASH_REVERIFICAR = Array(32).fill(0).map((_, i) => (i * 13) % 256);

// PDAs
const [registroPda] = PublicKey.findProgramAddressSync(
  [Buffer.from("registro_global")],
  pg.PROGRAM_ID
);
const [selloPda] = PublicKey.findProgramAddressSync(
  [Buffer.from("sello"), Buffer.from(ID_CEDULA)],
  pg.PROGRAM_ID
);

console.log("Programa:", pg.PROGRAM_ID.toBase58());
console.log("RegistroGlobal PDA:", registroPda.toBase58());
console.log("SelloCedula PDA:   ", selloPda.toBase58());

// 1) Inicializar (solo si aún no existe)
try {
  const tx = await pg.program.methods
    .inicializarRegistro()
    .accounts({
      admin: pg.wallet.publicKey,
      registro: registroPda,
      systemProgram: SystemProgram.programId,
    })
    .rpc();
  console.log("\n[OK] inicializar_registro tx:", tx);
} catch (e: any) {
  console.log("\n[skip] registro ya inicializado:", e.message?.slice(0, 80));
}

// 2) Sellar cédula
const txSellar = await pg.program.methods
  .sellarCedula(
    ID_CEDULA,
    FAKE_HASH_PAYLOAD,
    FAKE_HASH_NOMBRE,
    { nutricion: {} } // enum TipoProfesion::Nutricion
  )
  .accounts({
    operador: pg.wallet.publicKey,
    registro: registroPda,
    sello: selloPda,
    systemProgram: SystemProgram.programId,
  })
  .rpc();
console.log("\n[OK] sellar_cedula tx:", txSellar);

// 3) Re-verificar
const txReverif = await pg.program.methods
  .reVerificarCedula(FAKE_HASH_REVERIFICAR)
  .accounts({
    operador: pg.wallet.publicKey,
    registro: registroPda,
    sello: selloPda,
  })
  .rpc();
console.log("\n[OK] re_verificar_cedula tx:", txReverif);

// 4) Consultar (lectura on-chain via Account fetch)
const sello = await pg.program.account.selloCedula.fetch(selloPda);
console.log("\nEstado del sello on-chain:", {
  id_cedula: sello.idCedula,
  estatus: sello.estatus,
  tipoProfesion: sello.tipoProfesion,
  verificadaPor: sello.verificadaPor.toBase58(),
  slot: sello.slotVerificacion.toString(),
  contadorReverificaciones: sello.contadorReverificaciones,
});

// 5) Revocar
const txRevocar = await pg.program.methods
  .revocarCedula()
  .accounts({
    operador: pg.wallet.publicKey,
    registro: registroPda,
    sello: selloPda,
  })
  .rpc();
console.log("\n[OK] revocar_cedula tx:", txRevocar);

const selloFinal = await pg.program.account.selloCedula.fetch(selloPda);
console.log("\nEstatus final:", selloFinal.estatus); // -> { revocada: {} }

// Stats globales
const reg = await pg.program.account.registroGlobal.fetch(registroPda);
console.log("\nStats globales:", {
  totalSellos: reg.totalSellos.toString(),
  operadores: reg.operadores.length,
});
