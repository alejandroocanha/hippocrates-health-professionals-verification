# Hippocrates · Verificador On-Chain de Cédulas Profesionales del Sector Salud

> Proyecto final del bootcamp **[Solana Developer Certification — WayLearn LATAM](https://waylearn.gitbook.io/solana-developer-certification)**.
> Programa Anchor + Rust + Solana Playground · Devnet · CRUD + PDA.

`Hippocrates` es un programa de Solana que **sella on-chain** la verificación de cédulas profesionales del sector salud en México (medicina, odontología, psicología, nutrición, enfermería, etc.). Permite que un operador autorizado consulte el [Registro Nacional de Profesionistas (RENAPRO)](https://www.cedulaprofesional.sep.gob.mx/) y deje en cadena un **hash criptográfico** del estado verificado, junto con el slot, el timestamp y la firma del operador. Cualquier paciente, farmacia, hospital o aseguradora puede después leer el estado on-chain y confirmar que la cédula estaba vigente en el momento del sello.

---

## Tabla de contenidos

- [Cómo funciona (en una imagen)](#cómo-funciona-en-una-imagen)
- [Estructura del repo](#estructura-del-repo)
- [Instrucciones del programa](#instrucciones-del-programa)
- [PDAs](#pdas)
- [Cómo usarlo](#cómo-usarlo)
- [Ejemplo de salida del cliente](#ejemplo-de-salida-del-cliente)
- [Helper opcional `sep_query.ts`](#helper-opcional-sep_queryts)
- [Privacidad y consideraciones legales](#privacidad-y-consideraciones-legales)
- [Roadmap](#roadmap)
- [Licencia](#licencia)

---

## Cómo funciona (en una imagen)

```
Operador autorizado
    │ (1) consulta RENAPRO con scripts/sep_query.ts → JSON oficial
    │ (2) calcula sha256(payload normalizado) → hash 32 bytes
    ▼
Solana Devnet · programa Hippocrates
    ├── PDA RegistroGlobal (config + lista de operadores)
    └── PDA SelloCedula(idCedula)
            • hash_payload   (32 bytes)
            • estatus        (Vigente / Revocada / Pendiente)
            • tipo_profesion (Medicina | Odontologia | Psicologia | Nutricion | Enfermeria | Otro)
            • verificada_por (Pubkey del operador)
            • slot, timestamp del Clock sysvar

Tercero (paciente, farmacia, aseguradora)
    └── fetch(SelloCedula PDA) → ve el estatus
    └── recalcula sha256(payload off-chain) y compara con hash_payload on-chain
```

**No hay backend ni base de datos.** Toda la fuente de verdad vive on-chain. El servicio web del SEP solo se consulta off-chain por el operador para producir el hash de entrada.

---

## Estructura del repo

```
SOLANA-HIPPOCRATES/
├── src/
│   └── lib.rs              ← programa Anchor (Rust)
├── client/
│   └── client.ts           ← cliente TS para Solana Playground
├── scripts/
│   └── sep_query.ts        ← (opcional) consulta SEP + hash off-chain
├── tests/
│   └── anchor.test.ts      ← (opcional) tests Mocha
├── Anchor.toml             ← lo genera Solana Playground al hacer build
├── Cargo.toml              ← idem
└── README.md               ← este archivo
```

---

## Instrucciones del programa

| # | Instrucción | Tipo CRUD | Quién puede llamarla |
|---|---|---|---|
| 1 | `inicializar_registro` | Create (one-shot) | Cualquier wallet (queda como admin y primer operador) |
| 2 | `agregar_operador(pubkey)` | Update | Solo `admin` |
| 3 | `alternar_pausa()` | Update | Solo `admin` |
| 4 | `sellar_cedula(id, hash_payload, hash_nombre, tipo)` | Create | Operador autorizado |
| 5 | `re_verificar_cedula(nuevo_hash)` | Update | Operador autorizado |
| 6 | `revocar_cedula()` | Delete (soft) | Operador autorizado |
| 7 | `consultar_cedula()` | Read (log) | Cualquier wallet |

> Lectura programática real desde clientes: `program.account.selloCedula.fetch(selloPda)`.

### Errores definidos

```
NoEresAdmin · NoEresOperador · OperadorYaExiste · LimiteOperadoresAlcanzado
IdCedulaInvalido · ProgramaPausado · CedulaRevocada · OverflowContador
```

---

## PDAs

### `RegistroGlobal`

- **Seeds:** `[b"registro_global"]`
- **Campos:** `admin`, `operadores: Vec<Pubkey>` (max 10), `total_sellos`, `pausa_global`, `bump`.

### `SelloCedula`

- **Seeds:** `[b"sello", id_cedula.as_bytes()]` (una PDA por número de cédula).
- **Campos:** `id_cedula`, `hash_payload`, `nombre_completo_hash`, `tipo_profesion`, `estatus`, `verificada_por`, `slot_verificacion`, `unix_verificacion`, `contador_reverificaciones`, `bump`.

---

## Cómo usarlo

### Opción 1 — Solana Playground (más simple)

1. Abrir en el navegador:
   ```
   https://beta.solpg.io/https://github.com/alejandroocanha/SOLANA-HIPPOCRATES
   ```
2. En la esquina inferior izquierda, **crear o conectar wallet** (Devnet, con airdrop automático).
3. Terminal del Playground:
   ```
   build
   deploy
   client
   ```
4. Ver los logs de las 5 instrucciones ejecutándose y los `tx signatures` con link a Solana Explorer.

### Opción 2 — Local (Anchor CLI ≥ 0.31)

```bash
git clone https://github.com/alejandroocanha/SOLANA-HIPPOCRATES.git
cd SOLANA-HIPPOCRATES
anchor build
anchor deploy --provider.cluster devnet
ts-node client/client.ts
```

---

## Ejemplo de salida del cliente

```
Programa: 4Wq3...DPa
RegistroGlobal PDA: 7sxJ...rRn
SelloCedula PDA:    8dGL...mY9

[OK] inicializar_registro tx: 5xZ4...wY3
[OK] sellar_cedula tx:        2Bn7...tHr
[OK] re_verificar_cedula tx:  9Pq2...sAm
Estado del sello on-chain: {
  id_cedula: '9876543',
  estatus: { vigente: {} },
  tipoProfesion: { nutricion: {} },
  verificadaPor: 'GxRf...Qrt',
  slot: '356491230',
  contadorReverificaciones: 1
}
[OK] revocar_cedula tx: 4Lm8...kWp
Estatus final: { revocada: {} }
Stats globales: { totalSellos: '1', operadores: 1 }
```

---

## Helper opcional `sep_query.ts`

Para **producir el hash real** desde el RENAPRO antes de mandar la transacción:

```bash
ts-node scripts/sep_query.ts 1234567
```

Imprime el `id_cedula`, el `hash_payload` (32 bytes), el `nombre_completo_hash` (32 bytes) y el `tipo_profesion` deducido, listos para pegarse como argumentos de `sellar_cedula()`.

> **Nota:** No existe API REST oficial del gobierno mexicano para el RENAPRO. El script consulta el índice **Apache Solr público** que sirve internamente al portal `cedulaprofesional.sep.gob.mx`:
> ```
> GET http://search.sep.gob.mx/solr/cedulasCore/select?fl=*,score&q=idCedula:<NUMERO>&rows=1&wt=json
> ```
> Para producción se recomienda migrar a un proveedor con SLA (APIMarket, Nubarium, VerificaID).

---

## Privacidad y consideraciones legales

- **PII off-chain.** On-chain solo viven hashes (`SHA-256` de 32 bytes); el nombre y los datos personales nunca se persisten en claro.
- **LFPDPPP.** Los datos del RENAPRO son públicos por mandato del Art. 21 de la Ley Reglamentaria del Art. 5° Constitucional, pero aún así esta implementación minimiza exposición.
- **Derechos ARCO.** El admin puede ejecutar `revocar_cedula` ante una solicitud del titular.

---

## Roadmap

1. Frontend mínimo con `@solana/wallet-adapter` + un QR Solana Action que cualquier paciente pueda escanear.
2. Multi-attestation: que un sello requiera la firma de N operadores (Squads-style).
3. NFT comprimido (cNFT) por profesional verificado, llevado en su wallet como insignia.
4. Indexación con Helius / The Graph para dashboards públicos por especialidad.
5. Migración del proxy SEP a Switchboard / Pyth (oráculo descentralizado).

---

## Referencias

- Solana Developer Certification (WayLearn): <https://waylearn.gitbook.io/solana-developer-certification>
- Plantilla `WayLearnLatam/Biblioteca-Solana`: <https://github.com/WayLearnLatam/Biblioteca-Solana>
- Anchor Lang Book: <https://www.anchor-lang.com/>
- Solana Playground: <https://beta.solpg.io>
- Portal oficial Cédula Profesional SEP: <https://www.cedulaprofesional.sep.gob.mx/>
- Apache Solr — Common Query Parameters: <https://solr.apache.org/guide/solr/latest/query-guide/common-query-parameters.html>

---

## Licencia

MIT — ver `LICENSE` para detalles.
