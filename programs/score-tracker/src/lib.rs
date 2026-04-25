// =============================================================================
//  Score Tracker - Programa Solana / Anchor
//  Gaming
//
//  Cada jugador tiene un PDA derivado de:  [b"profile", authority.key()]
//  El PDA almacena su username, score acumulado y partidas jugadas.
//
//  Instrucciones (CRUD):
//    - create_player    -> CREATE  (init de la cuenta PDA)
//    - submit_score     -> UPDATE  (suma puntos a una partida)
//    - update_username  -> UPDATE  (cambia el nombre)
//    - delete_player    -> DELETE  (cierra el PDA y devuelve el rent)
//
//  La operacion READ se hace desde el cliente con
//  `program.account.playerProfile.fetch(pda)`, lo cual es el patron
//  estandar en Anchor (no necesita instruccion on-chain).
// =============================================================================

use anchor_lang::prelude::*;

declare_id!("8ySasPZ2NAcSM1w4KPzFg3AFZdkhBx381FJPvKqQTT39");

#[program]
pub mod score_tracker {
    use super::*;

    /// CREATE: inicializa el perfil del jugador.
    /// El PDA se deriva de la pubkey del firmante, asi cada wallet
    /// solo puede tener un perfil.
    pub fn create_player(ctx: Context<CreatePlayer>, username: String) -> Result<()> {
        require!(!username.is_empty(), ScoreError::UsernameEmpty);
        require!(username.len() <= 32, ScoreError::UsernameTooLong);

        let profile = &mut ctx.accounts.profile;
        profile.authority = ctx.accounts.authority.key();
        profile.username = username;
        profile.score = 0;
        profile.games_played = 0;
        profile.bump = ctx.bumps.profile;

        msg!("Perfil creado para: {}", profile.username);
        Ok(())
    }

    /// UPDATE: registra una partida sumando `points` al score
    /// e incrementa el contador de partidas jugadas.
    pub fn submit_score(ctx: Context<UpdatePlayer>, points: u64) -> Result<()> {
        let profile = &mut ctx.accounts.profile;

        profile.score = profile
            .score
            .checked_add(points)
            .ok_or(ScoreError::Overflow)?;

        profile.games_played = profile
            .games_played
            .checked_add(1)
            .ok_or(ScoreError::Overflow)?;

        msg!(
            "Score actualizado: {} (partidas: {})",
            profile.score,
            profile.games_played
        );
        Ok(())
    }

    /// UPDATE: permite al jugador renombrarse.
    pub fn update_username(ctx: Context<UpdatePlayer>, new_username: String) -> Result<()> {
        require!(!new_username.is_empty(), ScoreError::UsernameEmpty);
        require!(new_username.len() <= 32, ScoreError::UsernameTooLong);

        let profile = &mut ctx.accounts.profile;
        profile.username = new_username;

        msg!("Username actualizado a: {}", profile.username);
        Ok(())
    }

    /// DELETE: cierra el PDA y devuelve el rent (lamports) al authority.
    pub fn delete_player(_ctx: Context<DeletePlayer>) -> Result<()> {
        msg!("Perfil eliminado");
        Ok(())
    }
}

// -----------------------------------------------------------------------------
//  Contextos / cuentas requeridas para cada instruccion
// -----------------------------------------------------------------------------

#[derive(Accounts)]
#[instruction(username: String)]
pub struct CreatePlayer<'info> {
    #[account(
        init,
        payer = authority,
        space = PlayerProfile::LEN,
        seeds = [b"profile", authority.key().as_ref()],
        bump,
    )]
    pub profile: Account<'info, PlayerProfile>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdatePlayer<'info> {
    #[account(
        mut,
        seeds = [b"profile", authority.key().as_ref()],
        bump = profile.bump,
        has_one = authority @ ScoreError::Unauthorized,
    )]
    pub profile: Account<'info, PlayerProfile>,

    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct DeletePlayer<'info> {
    #[account(
        mut,
        close = authority,
        seeds = [b"profile", authority.key().as_ref()],
        bump = profile.bump,
        has_one = authority @ ScoreError::Unauthorized,
    )]
    pub profile: Account<'info, PlayerProfile>,

    #[account(mut)]
    pub authority: Signer<'info>,
}

// -----------------------------------------------------------------------------
//  Estado on-chain
// -----------------------------------------------------------------------------

#[account]
pub struct PlayerProfile {
    pub authority: Pubkey,    // 32  - dueno del perfil
    pub username: String,     // 4 + 32  - nombre publico
    pub score: u64,           // 8   - puntaje acumulado
    pub games_played: u64,    // 8   - partidas jugadas
    pub bump: u8,             // 1   - bump del PDA
}

impl PlayerProfile {
    // 8 (discriminator Anchor) + campos
    pub const LEN: usize = 8 + 32 + (4 + 32) + 8 + 8 + 1;
}

// -----------------------------------------------------------------------------
//  Errores personalizados
// -----------------------------------------------------------------------------

#[error_code]
pub enum ScoreError {
    #[msg("El username no puede estar vacio.")]
    UsernameEmpty,
    #[msg("El username supera los 32 caracteres permitidos.")]
    UsernameTooLong,
    #[msg("Overflow numerico al actualizar el score.")]
    Overflow,
    #[msg("Solo el dueno del perfil puede modificarlo.")]
    Unauthorized,
}