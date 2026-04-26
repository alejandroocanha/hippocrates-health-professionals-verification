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
