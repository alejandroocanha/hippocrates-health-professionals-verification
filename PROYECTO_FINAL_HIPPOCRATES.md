# Proyecto Final — Solana Developer Certification (WayLearn LATAM)

## Hippocrates · Verificador On-Chain de Cédulas Profesionales del Sector Salud

> **Autor:** Alejandro Ocaña Hernández — Ingeniero Civil (UNAM), Ingeniero en Sistemas Computacionales (UVEG), Nutriólogo (UVM)
> **Bootcamp:** Solana Developer Certification — WayLearn LATAM
> **Entrega:** 27 de abril de 2026, 13:00 hrs (CDMX) · [Formulario Airtable](https://airtable.com/appuy2jGnwR4b962V/pag9sPy79LlmZ14Pb/form)
> **Stack:** Solana Devnet · Anchor 0.31 · Rust · Solana Playground (`https://beta.solpg.io`) · Cliente TS de pruebas
> **Repo público:** `https://github.com/alejandroocanha/SOLANA-HIPPOCRATES`
> **Importable en Playground:** `https://beta.solpg.io/https://github.com/alejandroocanha/SOLANA-HIPPOCRATES`

---

## Índice

1. [Cumplimiento de la rúbrica oficial](#1-cumplimiento-de-la-rúbrica-oficial)
2. [Decisiones de diseño y simplicidad](#2-decisiones-de-diseño-y-simplicidad)
3. [Contexto y problema](#3-contexto-y-problema)
4. [Mapeo a las verticales sugeridas](#4-mapeo-a-las-verticales-sugeridas)
5. [Investigación: cómo se verifica una cédula profesional en México](#5-investigación-cómo-se-verifica-una-cédula-profesional-en-méxico)
6. [Arquitectura mínima](#6-arquitectura-mínima)
7. [Modelo de cuentas (PDA + struct)](#7-modelo-de-cuentas-pda--struct)
8. [Smart contract Anchor — código completo](#8-smart-contract-anchor--código-completo)
9. [Cliente de pruebas (Solana Playground)](#9-cliente-de-pruebas-solana-playground)
10. [Helper opcional: `sep_query.ts` para producir el hash desde RENAPRO](#10-helper-opcional-sep_queryts-para-producir-el-hash-desde-renapro)
11. [Estructura del repo y README](#11-estructura-del-repo-y-readme)
12. [Despliegue paso a paso (Solana Playground · Devnet)](#12-despliegue-paso-a-paso-solana-playground--devnet)
13. [Guion del Demo Day (5 minutos)](#13-guion-del-demo-day-5-minutos)
14. [Qué aprendí y qué sigue](#14-qué-aprendí-y-qué-sigue)
15. [Referencias](#15-referencias)

---

## 1. Cumplimiento de la rúbrica oficial

Tomando textualmente los **requisitos para la certificación** publicados por WayLearn LATAM (que pegaste en el chat) y mapeándolos uno a uno:

| Requisito oficial | Cómo lo cumple este proyecto |
|---|---|
| Repositorio público en GitHub | `github.com/alejandroocanha/SOLANA-HIPPOCRATES` |
| Proyecto libre | Identidad / credenciales del sector salud |
| Desarrollado en Solana (Rust + Anchor) | Programa Anchor 0.31 en Rust, importable a Solana Playground |
| **CRUD + PDA** | `Create` → `inicializar_registro` y `sellar_cedula`; `Read` → `consultar_cedula`; `Update` → `re_verificar_cedula`; `Delete` → `revocar_cedula` (soft delete on-chain). 2 PDAs: `RegistroGlobal` (seed `b"registro_global"`) y `SelloCedula` (seeds `[b"sello", id_cedula.as_bytes()]`). |
| Documentación: README o comentarios | `README.md` completo + comentarios `///` doc-style en cada `pub fn` y cada struct |
| Entrega vía formulario Airtable | URL del repo lista para pegar en el form |
| Demo Day: 5 min, en vivo | Sección 13: guion con las 5 partes (¿qué construiste? · inspiración · cómo funciona · qué aprendiste · qué sigue) |

---

## 2. Decisiones de diseño y simplicidad

> **Pregunta del autor:** ¿Vale la pena tener Supabase para presentar un proyecto final?
>
> **Respuesta razonada:** **No.** La rúbrica exige Solana (Rust + Anchor) con CRUD + PDA y un README. Agregar Supabase, Lovable, Edge Functions y bases de datos relacionales **no aporta puntos** y desvía la atención del programa Anchor —que es lo que realmente se evalúa— hacia plumbing accesorio. Para una demo de 5 minutos en vivo, simplicidad gana.

### Lo que **sí** entrega el MVP

- Programa Anchor en Devnet (`programs/hippocrates/src/lib.rs`).
- 5 instrucciones (CRUD completo + initialization).
- 2 PDAs derivadas (config global + por cédula).
- Cliente TypeScript de pruebas (`client/client.ts`) que se ejecuta dentro del Solana Playground —el mismo flujo del repo plantilla `Biblioteca-Solana`.
- `README.md` con descripción, instrucciones de uso, ejemplos y enlaces.
- Helper **opcional** (`scripts/sep_query.ts`) que muestra cómo cualquier verificador autorizado puede producir el hash a partir de la consulta real al Registro Nacional de Profesionistas. **No bloquea el programa**: el hash se genera off-chain y se pasa como argumento a `sellar_cedula`.

### Lo que **no** entrega el MVP (movido a "qué sigue")

- ❌ Backend Supabase / Edge Functions
- ❌ Frontend Lovable / React con wallet adapter
- ❌ NFT compresivo por profesional
- ❌ Oráculo descentralizado para la consulta a la SEP

Esto mantiene el alcance en lo que la rúbrica pide y permite un demo en vivo nítido en 5 minutos.

---

## 3. Contexto y problema

En México, el ejercicio profesional del sector salud (medicina, odontología, psicología, nutrición, enfermería, etc.) requiere **título registrado y cédula profesional vigente** ante la Dirección General de Profesiones (SEP), conforme al Art. 5° constitucional y a la Ley Reglamentaria del Art. 5°. El [Registro Nacional de Profesionistas](https://www.cedulaprofesional.sep.gob.mx/) lo publica, pero la consulta es **manual y no tiene API oficial**.

Resultado: pacientes, farmacias, aseguradoras y plataformas de telemedicina **no tienen un mecanismo automatizado, auditable y a prueba de manipulación** para confirmar que la cédula que un profesional muestra corresponde a un registro vigente. La suplantación es un problema documentado por COFEPRIS y PROFECO, especialmente en estética, nutrición y psicología.

**Hippocrates** ataca este problema con un programa Solana minimalista: anclar on-chain el `hash` del estado verificado de la cédula (verificado off-chain por un operador autorizado contra el RENAPRO), junto con la firma del operador, el slot y el timestamp. Cualquier tercero puede reconstruir el hash más tarde y obtener una **prueba criptográfica histórica** de que esa cédula estaba vigente en ese momento.

---

## 4. Mapeo a las verticales sugeridas

WayLearn lista verticales como inspiración (Tokens, Stablecoins, **DePIN**, Gaming, Creative, Payments, DeFi, **Solana Actions**). Este proyecto encaja en dos:

- **DePIN / Identidad descentralizada:** Hippocrates es infraestructura ligera de credenciales: cualquier paciente, hospital o aseguradora puede consultar el estado on-chain sin depender de un servidor centralizado. Encaja con el espíritu de "redes descentralizadas usando infraestructura del mundo real" aplicada al sector salud (la "infraestructura del mundo real" aquí es el ecosistema de profesionales certificados y los registros del RENAPRO).
- **Solana Actions (extensión natural):** una cédula sellada se puede exponer vía un link/QR `solana-action:` que permita a una farmacia o paciente verificar el estatus desde un código QR estampado en la receta o en el consultorio.

> No fuerzo la encuadre en DeFi ni Gaming porque sería artificial. La rúbrica permite proyecto libre y "creatividad", y el valor social aquí es claro.

---

## 5. Investigación: cómo se verifica una cédula profesional en México

> **Pregunta del autor:** ¿Cuál es la mejor forma de consultar programáticamente una cédula profesional en México? Buscando, no encuentro algo real.

### 5.1 Hallazgos

No existe API REST/SOAP **oficialmente documentada** del gobierno mexicano para el RENAPRO. Sin embargo, el portal `cedulaprofesional.sep.gob.mx` se sirve internamente sobre un **índice Apache Solr público** alojado en `search.sep.gob.mx`, que sí responde a `GET` y devuelve JSON. Es lo que han usado los wrappers comunitarios de GitHub durante años.

### 5.2 Endpoint de facto (no oficial pero estable)

```
GET http://search.sep.gob.mx/solr/cedulasCore/select
    ?fl=*,score
    &q=idCedula:<NUMERO>          ← o búsqueda libre por nombre
    &start=0&rows=1
    &facet=true&indent=on
    &wt=json
```

Parámetros relevantes ([Apache Solr Common Query Parameters](https://solr.apache.org/guide/solr/latest/query-guide/common-query-parameters.html)):

| Parámetro | Significado |
|---|---|
| `q` | Query. Búsqueda directa por número: `idCedula:1234567`. Por nombre: `andres+manuel+lopez+obrador`. |
| `fl` | Campos retornados; `*,score` regresa todo + score de relevancia. |
| `wt` | Writer del response; `json`. |
| `rows`, `start` | Paginación. |

Respuesta típica:

```json
{
  "responseHeader": { "status": 0, "QTime": 12 },
  "response": {
    "numFound": 1, "start": 0,
    "docs": [{
      "idCedula": "1234567",
      "nombre": "ALEJANDRO",
      "paterno": "OCAÑA",
      "materno": "HERNÁNDEZ",
      "tipoCedula": "FEDERAL",
      "nombreCarrera": "LICENCIATURA EN NUTRICIÓN",
      "nombreInstitucion": "UNIVERSIDAD DEL VALLE DE MÉXICO",
      "tipoTitulo": "DIPLOMA",
      "anioRegistro": "2010"
    }]
  }
}
```

### 5.3 Comparativa de opciones

| Opción | Costo | Confiabilidad | Recomendación |
|---|---|---|---|
| **Solr público SEP** | $0 | Media (sin SLA) | ✅ Para MVP / demo |
| Wrappers GitHub (`fmacias64/cedulas-sep-api`, `LuisEduardoHernandez/cedulas-de-la-sep-API`) | $0 | Baja | Solo como referencia |
| APIMarket / Nubarium / VerificaID (comerciales) | $$ | Alta, con SLA | Para producción futura |
| HTML scraping del portal | $0 | Frágil | ❌ Solr es superior |

### 5.4 Decisión

Para el demo: el operador (yo) ejecuta un script `sep_query.ts` que llama el Solr, normaliza la respuesta, calcula `sha256` y entrega el hash de 32 bytes que se pasa como argumento a `sellar_cedula`. **Todo el cómputo y el dato off-chain ocurren fuera de Solana**; on-chain solo viven hashes y metadatos no-sensibles. Esto reduce exposición bajo LFPDPPP (Ley Federal de Protección de Datos Personales en Posesión de los Particulares).

---

## 6. Arquitectura mínima

```
┌─────────────────┐       ┌────────────────────────┐       ┌────────────────────────┐
│  Operador       │  GET  │  search.sep.gob.mx      │       │  Solana Devnet         │
│  (Verificador   │──────▶│  solr/cedulasCore/select│       │  Programa Hippocrates  │
│   autorizado)   │       └────────────────────────┘       │  • RegistroGlobal PDA  │
│  • script TS    │              │ JSON                     │  • SelloCedula PDA     │
│  • Phantom      │              ▼                          │    (por cédula)        │
└─────────────────┘       ┌────────────────────────┐        └─────────┬──────────────┘
                          │ sep_query.ts            │                  │
                          │ • normaliza             │  tx Anchor       │
                          │ • sha256(payload_norm)  │  sellar_cedula() │
                          │ • imprime hash 32 bytes │──────────────────┘
                          └────────────────────────┘
                                                          ▲
                          Cualquier tercero ──────────────┤  Lectura pública vía
                          (paciente, farmacia,            │  program.account.selloCedula.fetch
                           aseguradora)                   │  o vía Solana Explorer
                                                          ▼
```

**Sin servidor backend. Sin frontend complejo.** Toda la "fuente de verdad" está on-chain. La consulta inicial al SEP es responsabilidad del operador y solo produce un input (`hash_payload`) para la transacción.

---

## 7. Modelo de cuentas (PDA + struct)

### 7.1 PDA `RegistroGlobal` (config + lista de operadores)

- **Seeds:** `[b"registro_global"]`
- **Bump:** almacenado en cuenta.
- **Campos:**

| Campo | Tipo | Notas |
|---|---|---|
| `admin` | `Pubkey` | Quien dio de alta el registro y puede agregar/quitar operadores |
| `operadores` | `Vec<Pubkey>` (max 10) | Wallets autorizadas a sellar |
| `total_sellos` | `u64` | Counter |
| `pausa_global` | `bool` | Kill-switch |
| `bump` | `u8` | |

### 7.2 PDA `SelloCedula` (uno por número de cédula)

- **Seeds:** `[b"sello", id_cedula.as_bytes()]`
- **Campos:**

| Campo | Tipo | Notas |
|---|---|---|
| `id_cedula` | `String` (max 16) | Número RENAPRO |
| `hash_payload` | `[u8; 32]` | SHA-256 del JSON normalizado de la SEP |
| `nombre_completo_hash` | `[u8; 32]` | SHA-256 del nombre normalizado (privacidad) |
| `tipo_profesion` | `TipoProfesion` (enum) | `Medicina`, `Odontologia`, `Psicologia`, `Nutricion`, `Enfermeria`, `Otro` |
| `estatus` | `EstatusCedula` (enum) | `Vigente`, `Revocada`, `PendienteReverificacion` |
| `verificada_por` | `Pubkey` | Operador que firmó |
| `slot_verificacion` | `u64` | `Clock::get()?.slot` |
| `unix_verificacion` | `i64` | `Clock::get()?.unix_timestamp` |
| `contador_reverificaciones` | `u16` | Cuántos refresh ha tenido |
| `bump` | `u8` | |

> **Decisión de privacidad:** No se persiste PII en claro. El nombre del profesional **solo se almacena como hash**. La PII en claro vive únicamente off-chain (en la respuesta del SEP que se descargó en el momento de sellar) y se puede reconstruir con `sha256(payload_off_chain) === hash_payload_on_chain`.

---

## 8. Smart contract Anchor — código completo

> Archivo: `programs/hippocrates/src/lib.rs`. Sigue exactamente la misma estructura comentada del repo plantilla `WayLearnLatam/Biblioteca-Solana` (declare_id vacío que el `build` rellena, `#[program]`, structs `#[derive(Accounts)]` por contexto, struct con `#[account]` y `#[derive(InitSpace)]`, enum `#[error_code]`).

```rust
use anchor_lang::prelude::*;

// ID del Solana Program. El "build" del Playground lo llena automáticamente.
declare_id!("");

// =====================================================================
// PROGRAMA: Hippocrates — Verificador de cédulas profesionales del sector salud
// CRUD + PDA en Solana Devnet | WayLearn LATAM Solana Developer Certification
// =====================================================================
#[program]
pub mod hippocrates {
    use super::*;

    // ───────────────────────────────────────────────────────────────────
    // (Create #1) Inicializar Registro Global
    // ───────────────────────────────────────────────────────────────────
    /// Crea la PDA de configuración global del programa.
    /// Quien la llama queda como `admin` y como primer operador.
    /// Se llama UNA sola vez por deploy.
    pub fn inicializar_registro(context: Context<InicializarRegistro>) -> Result<()> {
        let admin_id = context.accounts.admin.key();
        msg!("Inicializando registro global. Admin: {}", admin_id);

        let operadores: Vec<Pubkey> = vec![admin_id];

        context.accounts.registro.set_inner(RegistroGlobal {
            admin: admin_id,
            operadores,
            total_sellos: 0,
            pausa_global: false,
            bump: context.bumps.registro,
        });

        Ok(())
    }

    // ───────────────────────────────────────────────────────────────────
    // (Update) Agregar Operador
    // ───────────────────────────────────────────────────────────────────
    /// Solo el admin puede dar de alta nuevos operadores autorizados a sellar.
    pub fn agregar_operador(
        context: Context<ModificarRegistro>,
        nuevo_operador: Pubkey,
    ) -> Result<()> {
        let registro = &mut context.accounts.registro;
        require!(
            registro.admin == context.accounts.admin.key(),
            ErroresHippocrates::NoEresAdmin
        );
        require!(
            !registro.operadores.contains(&nuevo_operador),
            ErroresHippocrates::OperadorYaExiste
        );
        require!(
            registro.operadores.len() < 10,
            ErroresHippocrates::LimiteOperadoresAlcanzado
        );
        registro.operadores.push(nuevo_operador);
        msg!("Operador agregado: {}", nuevo_operador);
        Ok(())
    }

    // ───────────────────────────────────────────────────────────────────
    // (Update) Pausa global / kill-switch
    // ───────────────────────────────────────────────────────────────────
    pub fn alternar_pausa(context: Context<ModificarRegistro>) -> Result<()> {
        let registro = &mut context.accounts.registro;
        require!(
            registro.admin == context.accounts.admin.key(),
            ErroresHippocrates::NoEresAdmin
        );
        registro.pausa_global = !registro.pausa_global;
        msg!("Pausa global ahora: {}", registro.pausa_global);
        Ok(())
    }

    // ───────────────────────────────────────────────────────────────────
    // (Create #2) Sellar Cédula — instrucción principal
    // ───────────────────────────────────────────────────────────────────
    /// Crea (o falla si ya existe) la PDA SelloCedula con el hash del payload
    /// normalizado de la SEP. Solo operadores autorizados.
    pub fn sellar_cedula(
        context: Context<SellarCedula>,
        id_cedula: String,
        hash_payload: [u8; 32],
        nombre_completo_hash: [u8; 32],
        tipo_profesion: TipoProfesion,
    ) -> Result<()> {
        let registro = &mut context.accounts.registro;
        let operador = context.accounts.operador.key();

        require!(!registro.pausa_global, ErroresHippocrates::ProgramaPausado);
        require!(
            registro.operadores.contains(&operador),
            ErroresHippocrates::NoEresOperador
        );
        require!(
            !id_cedula.is_empty() && id_cedula.len() <= 16,
            ErroresHippocrates::IdCedulaInvalido
        );

        let clock = Clock::get()?;
        context.accounts.sello.set_inner(SelloCedula {
            id_cedula: id_cedula.clone(),
            hash_payload,
            nombre_completo_hash,
            tipo_profesion,
            estatus: EstatusCedula::Vigente,
            verificada_por: operador,
            slot_verificacion: clock.slot,
            unix_verificacion: clock.unix_timestamp,
            contador_reverificaciones: 0,
            bump: context.bumps.sello,
        });

        registro.total_sellos = registro
            .total_sellos
            .checked_add(1)
            .ok_or(ErroresHippocrates::OverflowContador)?;

        msg!("Cédula sellada: {} por operador {}", id_cedula, operador);
        Ok(())
    }

    // ───────────────────────────────────────────────────────────────────
    // (Update) Re-verificar Cédula
    // ───────────────────────────────────────────────────────────────────
    /// Refresca el hash y actualiza el timestamp/slot. No crea cuenta nueva.
    pub fn re_verificar_cedula(
        context: Context<ModificarSello>,
        nuevo_hash_payload: [u8; 32],
    ) -> Result<()> {
        let registro = &context.accounts.registro;
        let operador = context.accounts.operador.key();

        require!(!registro.pausa_global, ErroresHippocrates::ProgramaPausado);
        require!(
            registro.operadores.contains(&operador),
            ErroresHippocrates::NoEresOperador
        );

        let sello = &mut context.accounts.sello;
        require!(
            sello.estatus != EstatusCedula::Revocada,
            ErroresHippocrates::CedulaRevocada
        );

        let clock = Clock::get()?;
        sello.hash_payload = nuevo_hash_payload;
        sello.estatus = EstatusCedula::Vigente;
        sello.verificada_por = operador;
        sello.slot_verificacion = clock.slot;
        sello.unix_verificacion = clock.unix_timestamp;
        sello.contador_reverificaciones = sello
            .contador_reverificaciones
            .checked_add(1)
            .ok_or(ErroresHippocrates::OverflowContador)?;

        msg!(
            "Cédula {} re-verificada (#{}) por {}",
            sello.id_cedula,
            sello.contador_reverificaciones,
            operador
        );
        Ok(())
    }

    // ───────────────────────────────────────────────────────────────────
    // (Delete — soft) Revocar Cédula
    // ───────────────────────────────────────────────────────────────────
    /// Marca la cédula como Revocada. No cierra la cuenta para preservar
    /// el historial verificable on-chain.
    pub fn revocar_cedula(context: Context<ModificarSello>) -> Result<()> {
        let registro = &context.accounts.registro;
        let operador = context.accounts.operador.key();

        require!(
            registro.operadores.contains(&operador),
            ErroresHippocrates::NoEresOperador
        );

        let sello = &mut context.accounts.sello;
        let clock = Clock::get()?;
        sello.estatus = EstatusCedula::Revocada;
        sello.unix_verificacion = clock.unix_timestamp;
        sello.slot_verificacion = clock.slot;

        msg!("Cédula {} REVOCADA por {}", sello.id_cedula, operador);
        Ok(())
    }

    // ───────────────────────────────────────────────────────────────────
    // (Read) Consultar Cédula
    // ───────────────────────────────────────────────────────────────────
    /// Imprime en log el estado actual del sello. Para lectura programática
    /// los clientes deben usar `program.account.selloCedula.fetch(pda)`.
    pub fn consultar_cedula(context: Context<ConsultarSello>) -> Result<()> {
        let sello = &context.accounts.sello;
        msg!(
            "Cedula {:?} | estatus={:?} | profesion={:?} | re-verif={} | slot={}",
            sello.id_cedula,
            sello.estatus,
            sello.tipo_profesion,
            sello.contador_reverificaciones,
            sello.slot_verificacion
        );
        Ok(())
    }
}

// =====================================================================
// CUENTAS PERSISTIDAS ON-CHAIN
// =====================================================================

#[account]
#[derive(InitSpace)]
pub struct RegistroGlobal {
    pub admin: Pubkey,
    #[max_len(10)]
    pub operadores: Vec<Pubkey>,
    pub total_sellos: u64,
    pub pausa_global: bool,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct SelloCedula {
    #[max_len(16)]
    pub id_cedula: String,
    pub hash_payload: [u8; 32],
    pub nombre_completo_hash: [u8; 32],
    pub tipo_profesion: TipoProfesion,
    pub estatus: EstatusCedula,
    pub verificada_por: Pubkey,
    pub slot_verificacion: u64,
    pub unix_verificacion: i64,
    pub contador_reverificaciones: u16,
    pub bump: u8,
}

// =====================================================================
// ENUMS
// =====================================================================

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, Debug, InitSpace)]
pub enum TipoProfesion {
    Medicina,
    Odontologia,
    Psicologia,
    Nutricion,
    Enfermeria,
    Otro,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, Debug, InitSpace)]
pub enum EstatusCedula {
    Vigente,
    Revocada,
    PendienteReverificacion,
}

// =====================================================================
// CONTEXTOS DE INSTRUCCIONES
// =====================================================================

#[derive(Accounts)]
pub struct InicializarRegistro<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        init,
        payer = admin,
        space = RegistroGlobal::INIT_SPACE + 8,
        seeds = [b"registro_global"],
        bump
    )]
    pub registro: Account<'info, RegistroGlobal>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ModificarRegistro<'info> {
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [b"registro_global"],
        bump = registro.bump
    )]
    pub registro: Account<'info, RegistroGlobal>,
}

#[derive(Accounts)]
#[instruction(id_cedula: String)] // necesario para usar id_cedula como seed
pub struct SellarCedula<'info> {
    #[account(mut)]
    pub operador: Signer<'info>,

    #[account(
        mut,
        seeds = [b"registro_global"],
        bump = registro.bump
    )]
    pub registro: Account<'info, RegistroGlobal>,

    #[account(
        init,
        payer = operador,
        space = SelloCedula::INIT_SPACE + 8,
        seeds = [b"sello", id_cedula.as_bytes()],
        bump
    )]
    pub sello: Account<'info, SelloCedula>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ModificarSello<'info> {
    pub operador: Signer<'info>,

    #[account(
        seeds = [b"registro_global"],
        bump = registro.bump
    )]
    pub registro: Account<'info, RegistroGlobal>,

    #[account(
        mut,
        seeds = [b"sello", sello.id_cedula.as_bytes()],
        bump = sello.bump
    )]
    pub sello: Account<'info, SelloCedula>,
}

#[derive(Accounts)]
pub struct ConsultarSello<'info> {
    #[account(
        seeds = [b"sello", sello.id_cedula.as_bytes()],
        bump = sello.bump
    )]
    pub sello: Account<'info, SelloCedula>,
}

// =====================================================================
// ERRORES
// =====================================================================

#[error_code]
pub enum ErroresHippocrates {
    #[msg("Error: solo el admin puede ejecutar esta accion")]
    NoEresAdmin,
    #[msg("Error: tu wallet no es operador autorizado")]
    NoEresOperador,
    #[msg("Error: el operador ya esta registrado")]
    OperadorYaExiste,
    #[msg("Error: limite de operadores alcanzado (max 10)")]
    LimiteOperadoresAlcanzado,
    #[msg("Error: id_cedula vacio o mayor a 16 caracteres")]
    IdCedulaInvalido,
    #[msg("Error: el programa esta pausado")]
    ProgramaPausado,
    #[msg("Error: la cedula esta revocada y no admite re-verificacion")]
    CedulaRevocada,
    #[msg("Error: overflow en contador")]
    OverflowContador,
}
```

---

## 9. Cliente de pruebas (Solana Playground)

> Archivo: `client/client.ts`. Solana Playground inyecta automáticamente `pg.wallet`, `pg.connection`, `pg.PROGRAM_ID` y `pg.program`. Este script ejecuta el flujo CRUD completo y termina dejando trazas legibles en el log.

```ts
// client/client.ts — Hippocrates · CRUD demo en Devnet
import { PublicKey, SystemProgram } from "@solana/web3.js";

const ID_CEDULA = "9876543";

// Hash demo (en producción viene de scripts/sep_query.ts)
const FAKE_HASH_PAYLOAD       = Array(32).fill(0).map((_, i) => (i * 7) % 256);
const FAKE_HASH_NOMBRE        = Array(32).fill(0).map((_, i) => (i * 11) % 256);
const FAKE_HASH_REVERIFICAR   = Array(32).fill(0).map((_, i) => (i * 13) % 256);

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
```

> En el Playground basta con **abrir la pestaña Test → click en "Run"** o, desde la terminal, ejecutar `client`.

---

## 10. Helper opcional: `sep_query.ts` para producir el hash desde RENAPRO

> Archivo: `scripts/sep_query.ts`. **No** es parte del programa Anchor; es una utilidad para que el operador, en su laptop, produzca el `hash_payload` real antes de mandar la transacción. Ejecutable con `ts-node` o `bun`.

```ts
// scripts/sep_query.ts
// Uso: ts-node scripts/sep_query.ts 1234567
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
  const url = `${SEP}?fl=*,score&q=idCedula:${encodeURIComponent(idCedula)}&start=0&rows=1&facet=true&indent=on&wt=json`;
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

  // hash canónico (claves ordenadas)
  const canon = JSON.stringify(payload, Object.keys(payload).sort());
  const hashPayload = sha256(canon);
  const hashNombre  = sha256(`${payload.nombre}|${payload.paterno}|${payload.materno}`);

  console.log("Payload (off-chain, no se almacena en cadena):");
  console.log(payload);
  console.log("\n--- Inputs para sellar_cedula() ---");
  console.log("id_cedula            :", payload.id_cedula);
  console.log("hash_payload         : [", Array.from(hashPayload).join(", "), "]");
  console.log("nombre_completo_hash : [", Array.from(hashNombre).join(", "), "]");
  console.log("tipo_profesion       :", clasificar(payload.nombre_carrera));
})();
```

> El operador copia el resultado y lo pega en `client/client.ts` (o ejecuta una variante del cliente que tome los args directamente).

---

## 11. Estructura del repo y README

Estructura mínima del repo `SOLANA-HIPPOCRATES`:

```
SOLANA-HIPPOCRATES/
├── src/
│   └── lib.rs                     ← contenido de la sección 8
├── client/
│   └── client.ts                  ← contenido de la sección 9
├── scripts/
│   └── sep_query.ts               ← contenido de la sección 10 (opcional)
├── tests/
│   └── anchor.test.ts             ← (opcional) versión Mocha del client.ts
├── Anchor.toml                    ← lo genera Solana Playground al hacer "build"
├── Cargo.toml                     ← idem
└── README.md                      ← se incluye también en este repo
```

> El `README.md` del repo se entrega como archivo separado (`README.md` en la raíz del proyecto). Su contenido vive como anexo a este documento y reproduce los puntos esenciales: descripción, flujo, instrucciones de uso, screenshots y enlaces.

---

## 12. Despliegue paso a paso (Solana Playground · Devnet)

Mismo patrón que `WayLearnLatam/Biblioteca-Solana`:

1. **Crear repo** público `github.com/alejandroocanha/SOLANA-HIPPOCRATES` con la estructura de la sección 11.
2. **Importar** al Playground abriendo `https://beta.solpg.io/https://github.com/alejandroocanha/SOLANA-HIPPOCRATES`.
3. **Crear wallet temporal** (icono de wallet abajo a la izquierda → *Create new wallet*) → Devnet automáticamente con airdrop. Si hace falta más SOL: terminal → `solana airdrop 5`.
4. **Build** desde la terminal del Playground:
   ```
   build
   ```
   Anchor 0.31 autocompleta `declare_id!("...")` con la pubkey.
5. **Deploy**:
   ```
   deploy
   ```
6. **Ejecutar el cliente**:
   ```
   client
   ```
   Ver el log con las 5 instrucciones ejecutadas y los enlaces de Explorer.
7. **Capturar evidencia** del Explorer / Solscan para los slides del demo.

---

## 13. Guion del Demo Day (5 minutos)

> Estructura sugerida por WayLearn (que pegaste en el chat): qué construiste · inspiración · cómo funciona · qué aprendiste · qué sigue.

### Minuto 0:00 → 0:30 — ¿Qué construiste? (1 frase)

> "Hippocrates es un programa Solana que sella **on-chain** la verificación de cédulas profesionales del sector salud en México, para que cualquier paciente pueda confirmar criptográficamente que su médico, dentista, psicólogo o nutriólogo está realmente cedulado."

### Minuto 0:30 → 1:30 — Inspiración / problema

- Soy nutriólogo titulado, además de ingeniero. La **suplantación profesional** es un problema documentado por COFEPRIS y PROFECO, sobre todo en estética, nutrición y psicología.
- Hoy la consulta del **RENAPRO** (`cedulaprofesional.sep.gob.mx`) es manual, sin API oficial, y un paciente típico no la usa.
- ¿Y si el sello de "verifiqué tu cédula contra la SEP" estuviera en una blockchain pública, audible, inmutable, y consultable desde un QR?

### Minuto 1:30 → 3:30 — Cómo funciona (flujo)

```
Operador autorizado
    │ (1) consulta RENAPRO con sep_query.ts → obtiene JSON
    │ (2) calcula sha256(payload normalizado) → hash 32 bytes
    ▼
Solana Devnet · programa Hippocrates
    ├── PDA RegistroGlobal  (config + lista de operadores)
    └── PDA SelloCedula(idCedula)  ← una por cédula
            • hash_payload      (32 bytes)
            • estatus           (Vigente / Revocada / Pendiente)
            • tipo_profesion    (Medicina, Odontologia, Psicologia, Nutricion, ...)
            • verificada_por    (Pubkey del operador)
            • slot, timestamp   (Clock sysvar)

Cualquier tercero → fetch de la PDA → ve el estatus + verifica que sha256 coincide.
```

**Demo en vivo:**
1. Compartir pantalla con el Solana Playground abierto.
2. Mostrar `lib.rs` con las 5 instrucciones (subir/bajar comentando los `pub fn`).
3. Ejecutar `client` y enseñar el log: tx signatures de `inicializar_registro → sellar_cedula → re_verificar_cedula → revocar_cedula`.
4. Click en una signature → abrir Solana Explorer en pestaña → enseñar la cuenta `SelloCedula` cambiando de `vigente` a `revocada`.

### Minuto 3:30 → 4:30 — Qué aprendí (1-2 cosas técnicas)

- **PDAs con seeds dinámicas (`#[instruction(id_cedula: String)]`):** una PDA por cédula me obliga a entender bien por qué Anchor necesita el atributo `#[instruction(...)]` cuando un argumento de la instrucción se usa como semilla — antes lo pasaba por encima.
- **Diseño de privacidad on-chain:** descubrí que es trivial almacenar PII en Solana, y precisamente por eso decidí guardar **solo hashes** (PII fuera de cadena). Esto se alinea con LFPDPPP en México y mantiene el contrato barato (rent-exempt ≈ 0.0014 SOL por cédula).

### Minuto 4:30 → 5:00 — Qué sigue

Si tuviera una semana más:
1. Frontend mínimo con `@solana/wallet-adapter` + un QR de Solana Action que cualquier paciente pueda escanear desde su celular.
2. Multi-attestation: que un sello requiera la firma de N operadores (Squads-style).
3. Notificaciones via Helius webhooks cuando una cédula cambia a `Revocada`.

---

## 14. Qué aprendí y qué sigue

### 14.1 Aprendizajes técnicos

- Anchor 0.31: macros (`#[program]`, `#[derive(Accounts)]`, `#[account]`, `#[error_code]`, `#[derive(InitSpace)]`), uso de `space = X::INIT_SPACE + 8`, derivación de PDAs con `seeds` y `bump`, validación de signers con `require!`.
- Solana Playground: workflow GitHub → import → build → deploy en navegador, sin tooling local.
- Diseño de cuentas: cuándo usar enum vs string, hashing canónico off-chain (claves ordenadas) para que el resultado sea reproducible.
- Privacidad: hashes en lugar de PII, alineado con LFPDPPP.

### 14.2 Aprendizajes de producto

- La rúbrica de un bootcamp es lo opuesto a un MVP de startup: gana lo simple bien explicado. Retiré Supabase, Lovable y NFTs del scope inicial porque no aportaban a la evaluación.
- "Demo en vivo" es muy distinto de "demo grabado": cada componente del stack agrega un punto de falla potencial frente a los jueces.

### 14.3 Roadmap post-bootcamp

1. Frontend con wallet adapter + QR Solana Actions.
2. NFT comprimido (cNFT) por profesional sellado, para que el médico lleve la insignia en su wallet.
3. Multi-attestation con varios operadores (colegios profesionales, hospitales).
4. Indexación con Helius / The Graph para dashboards públicos por especialidad.
5. Migración del proxy SEP a Switchboard / Pyth (oráculo descentralizado con varias attestations).

---

## 15. Referencias

### Bootcamp WayLearn

- Solana Developer Certification — WayLearn LATAM: <https://waylearn.gitbook.io/solana-developer-certification>
- Sesiones interactivas (Demo Day): <https://waylearn.gitbook.io/solana-developer-certification/sesiones-interactivas>
- WayLearnLatam/Biblioteca-Solana (plantilla CRUD + PDA): <https://github.com/WayLearnLatam/Biblioteca-Solana>
- WayLearnLatam/La-Poderosa-Biblioteca-Solana: <https://github.com/WayLearnLatam/La-Poderosa-Biblioteca-Solana>
- WayLearnLatam/Solana-starter-kit: <https://github.com/WayLearnLatam/Solana-starter-kit>
- WayLearnLatam/Taller-Frontend-Solana: <https://github.com/WayLearnLatam/Taller-Frontend-Solana>
- WayLearnLatam/Solana-Hackathon-Template-FullStack: <https://github.com/WayLearnLatam/Solana-Hackathon-Template-FullStack>
- WayLearnLatam/Solana-Hackathon-Template-Backend: <https://github.com/WayLearnLatam/Solana-Hackathon-Template-Backend>
- WayLearnLatam/Awesome-Solana-WayLearn: <https://github.com/WayLearnLatam/Awesome-Solana-WayLearn>
- WayLearnLatam/Learning-Rust: <https://github.com/WayLearnLatam/Learning-Rust>

### Solana / Anchor

- Solana Docs: <https://solana.com/docs>
- Anchor Lang Book: <https://www.anchor-lang.com/>
- Solana Playground: <https://beta.solpg.io>
- Solana Cookbook (PDAs): <https://solanacookbook.com/core-concepts/pdas.html>
- Solana Actions: <https://solana.com/docs/advanced/actions>

### Cédula profesional / SEP / RENAPRO

- Portal oficial — Cédula Profesional SEP: <https://www.cedulaprofesional.sep.gob.mx/>
- SIURP — Sistema de Cédulas Profesionales: <https://siurp.sep.gob.mx/mvc/cedulaElectronica>
- fmacias64/cedulas-sep-api (referencia, wrapper público): <https://github.com/fmacias64/cedulas-sep-api>
- LuisEduardoHernandez/cedulas-de-la-sep-API (endpoint Solr documentado): <https://github.com/LuisEduardoHernandez/cedulas-de-la-sep-API>
- rchargoy/consulta-cedula-profesional (iOS app de referencia): <https://github.com/rchargoy/consulta-cedula-profesional>
- Apache Solr — Common Query Parameters: <https://solr.apache.org/guide/solr/latest/query-guide/common-query-parameters.html>

### Marco legal mexicano

- Ley Reglamentaria del Art. 5° Constitucional (Ley de Profesiones).
- Ley Federal de Protección de Datos Personales en Posesión de los Particulares (LFPDPPP), DOF.
- COFEPRIS — alertas de suplantación de profesionales de la salud.

---

> **Nota de transparencia para el evaluador:** El alcance del MVP se redujo deliberadamente a lo que pide la rúbrica oficial (Anchor + CRUD + PDA + README) tras revisar la sección "Sesiones Interactivas → Día 7 Demo Day" del GitBook. Las extensiones (frontend, backend, NFT, multi-attestation) están enumeradas en *Qué sigue* como roadmap de producto, no como entregables del bootcamp.
