# Hippocrates

**Verificador on-chain de cédulas profesionales del sector salud en México.**

Este proyecto sellla en Solana la verificación de cédulas de médicos, nutriólogos, psicólogos, dentistas y otros profesionistas del salud. El objetivo es que cualquier persona pueda confirmar criptográficamente que una cédula estaba vigente en un momento dado, sin depender de un servidor central.

---

## Por qué existe este proyecto

En México, ejercer como profesionista de la salud sin cédula válida es un problema real. COFEPRIS y PROFECO han documentado casos de suplantación profesional, sobre todo en áreas como estética, nutrición y psicología.

Hoy la consulta al RENAPRO es manual y no tiene API oficial. Hippocrates propone usar la blockchain de Solana como testigo inmutable: un operador autorizado verifica contra el registro oficial, calcula un hash, y lo sella on-chain junto con el timestamp y su firma digital.

Esto no reemplaza al RENAPRO — lo usa como fuente de verdad off-chain y ancla el resultado en Solana.

---

## Estructura del proyecto

```
SOLANA-HIPPOCRATES/
├── src/
│   └── lib.rs          ← Programa Anchor (Rust). Acá viven las cuentas,
│                        los contextos y las 5 instrucciones.
├── client/
│   └── client.ts       ← Script de pruebas para Solana Playground.
│                        Ejecuta todo el flujo CRUD de una vez.
├── scripts/
│   └── sep_query.ts    ← Helper que consulta el Solr público de la SEP
│                        y produce los hashes para sellar_cedula().
├── tests/
│   └── anchor.test.ts  ← Tests (no obligatorios para la entrega).
└── README.md
```

---

## Qué hace cada carpeta

**`src/`** — El smart contract en Rust. Define dos cuentas:
- `RegistroGlobal`: configuración del programa y lista de operadores autorizados.
- `SelloCedula`: una por cada número de cédula sellado. Contiene el hash, el estatus y metadatos de verificación.

**`client/`** — Para probar sin frontend. Se corre directo en el Playground después de `deploy`.

**`scripts/`** — Helper opcional. No es parte del programa on-chain; sirve para que el operador produzca el hash real desde la respuesta de la SEP antes de llamar a `sellar_cedula`.

---

## Estado actual

Esto es un MVP funcional para el bootcamp. Puse a andar el programa completo en Devnet y verificaste que el flujo funciona:

```
inicializar_registro → sellar_cedula → re_verificar_cedula → consultar → revocar
```

Pero no tiene interfaz gráfica, no está en mainnet, y el script de SEP usa un endpoint público sin SLA.

---

## Próximos pasos (backlog)

Si esto siguiera adelante, las cosas que haría:

1. **Frontend** — Una página mínima donde el usuario meta el número de cédula y vea el resultado. Probablemente con React y el wallet adapter de Phantom.

2. **QR de Solana Action** — Que cualquier paciente escanee un código QR en la receta o en el consultorio y pueda verificar al instante desde su celular.

3. **Multi-attestation** — Que un sello requiera la firma de 2 o 3 operadores (por ejemplo, el colegio profesional + el hospital). Estilo multisig.

4. **cNFT por profesionista** — Una vez sellada la cédula, mintear un token comprimido en la wallet del profesionista como insignia verificable.

5. **Oráculo descentralizado** — Reemplazar el script que consulta al Solr de la SEP por un oráculo como Switchboard o Pyth, para tener varias fuentes y no depender de un solo endpoint.

6. **Dashboard público** — Indexar con Helius o The Graph para mostrar estadísticas por especialidad, por estado, por institución.

---

## Cómo correrlo hoy

### Solana Playground (lo más rápido)

1. Abrir: `https://beta.solpg.io/https://github.com/alejandroocanha/hippocrates-health-professionals-verification`
2. Conectar wallet (crear una nueva o Phantom)
3. En terminal:
   ```
   build
   deploy
   client
   ```

### Local (requiere Anchor CLI)

```bash
git clone https://github.com/alejandroocanha/hippocrates-health-professionals-verification.git
cd hippocrates-health-professionals-verification
anchor build
anchor deploy --provider.cluster devnet
ts-node client/client.ts
```

---

## Referencias

- [Bootcamp Solana Developer — WayLearn LATAM](https://waylearn.gitbook.io/solana-developer-certification)
- [Plantilla Biblioteca-Solana](https://github.com/WayLearnLatam/Biblioteca-Solana)
- [Anchor Lang Book](https://www.anchor-lang.com/)
- [Solana Playground](https://beta.solpg.io)
- [Portal Cédula Profesional SEP](https://www.cedulaprofesional.sep.gob.mx/)

---

*Proyecto desarrollado como entregable final del bootcamp Solana Developer Certification — WayLearn LATAM, Abril 2026.*