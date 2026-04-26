// scripts/sep_query.ts
// Helper opcional: consulta el RENAPRO (Solr público de la SEP), normaliza la
// respuesta y produce los hashes de 32 bytes listos para sellar_cedula().
//
// Uso:
//   ts-node scripts/sep_query.ts <idCedula>
//   bun run scripts/sep_query.ts <idCedula>
//
// No es parte del programa Anchor: corre en la laptop del operador autorizado.

import { createHash } from "crypto";

const SEP = "http://search.sep.gob.mx/solr/cedulasCore/select";

interface CedulaPayload {
  id_cedula: string;
  nombre: string;
  paterno: string;
  materno: string;
  tipo_cedula: string;
  nombre_carrera: string;
  nombre_institucion: string;
  tipo_titulo: string;
  anio_registro: string;
}

function normalizar(s: string): string {
  return (s ?? "")
    .normalize("NFD")
    .replace(/[̀-ͯ]/g, "")
    .toUpperCase()
    .trim()
    .replace(/\s+/g, " ");
}

async function consultar(idCedula: string): Promise<CedulaPayload | null> {
  const url = `${SEP}?fl=*,score&q=idCedula:${encodeURIComponent(
    idCedula
  )}&start=0&rows=1&facet=true&indent=on&wt=json`;

  const r = await fetch(url);
  if (!r.ok) return null;
  const json: any = await r.json();
  const d = json?.response?.docs?.[0];
  if (!d) return null;
  return {
    id_cedula: String(d.idCedula ?? "").trim(),
    nombre: normalizar(d.nombre),
    paterno: normalizar(d.paterno),
    materno: normalizar(d.materno),
    tipo_cedula: normalizar(d.tipoCedula),
    nombre_carrera: normalizar(d.nombreCarrera),
    nombre_institucion: normalizar(d.nombreInstitucion),
    tipo_titulo: normalizar(d.tipoTitulo),
    anio_registro: String(d.anioRegistro ?? "").trim(),
  };
}

function clasificar(carrera: string): string {
  const c = carrera.toUpperCase();
  if (c.includes("MEDICINA") || c.includes("MEDIC")) return "Medicina";
  if (c.includes("ODONT") || c.includes("DENT"))     return "Odontologia";
  if (c.includes("PSIC"))                            return "Psicologia";
  if (c.includes("NUTRI"))                           return "Nutricion";
  if (c.includes("ENFERM"))                          return "Enfermeria";
  return "Otro";
}

function sha256(s: string): Buffer {
  return createHash("sha256").update(s).digest();
}

(async () => {
  const idCedula = process.argv[2];
  if (!idCedula) {
    console.error("Uso: ts-node scripts/sep_query.ts <idCedula>");
    process.exit(1);
  }

  const payload = await consultar(idCedula);
  if (!payload) {
    console.error("No se encontró la cédula o el endpoint no respondió.");
    process.exit(2);
  }

  // Hash canónico (claves ordenadas) → resultado reproducible
  const canon = JSON.stringify(payload, Object.keys(payload).sort());
  const hashPayload = sha256(canon);
  const hashNombre  = sha256(`${payload.nombre}|${payload.paterno}|${payload.materno}`);

  console.log("\nPayload (off-chain, no se almacena en cadena):");
  console.log(payload);

  console.log("\n--- Inputs para sellar_cedula() ---");
  console.log("id_cedula            :", payload.id_cedula);
  console.log("hash_payload         : [", Array.from(hashPayload).join(", "), "]");
  console.log("nombre_completo_hash : [", Array.from(hashNombre).join(", "), "]");
  console.log("tipo_profesion       :", clasificar(payload.nombre_carrera));
})();
