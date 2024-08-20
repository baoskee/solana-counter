use anchor_lang::prelude::*;
use anchor_spl::metadata::{
    create_metadata_accounts_v3, mpl_token_metadata::types::DataV2, CreateMetadataAccountsV3,
};
use anchor_spl::{
    associated_token::AssociatedToken,
    metadata::Metadata,
    token::{Mint, Token, TokenAccount},
};

declare_id!("2AvdcjV1eA45F98Uo1iN6CDG8QThTUPi7Rmn6nHSCET6");

#[program]
pub mod spl_demo {

    use anchor_spl::token::{mint_to, MintTo};

    use super::*;

    // 1. create token with metadata
    // 2. mint `mint_a` tokens to vault
    pub fn initialize(
        ctx: Context<Initialize>,
        token_name: String,
        token_symbol: String,
        token_uri: String,
        mint_amount: u64, // in lamports
    ) -> Result<()> {
        // 1. Invoking the create_metadata_account_v3 instruction on the token metadata program
        let cpi_context = CpiContext::new(
            ctx.accounts.token_metadata_program.to_account_info(),
            CreateMetadataAccountsV3 {
                metadata: ctx.accounts.metadata_a.to_account_info(),
                mint: ctx.accounts.mint_a.to_account_info(),
                mint_authority: ctx.accounts.payer.to_account_info(),
                update_authority: ctx.accounts.payer.to_account_info(),
                payer: ctx.accounts.payer.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                rent: ctx.accounts.rent.to_account_info(),
            },
        );

        create_metadata_accounts_v3(
            cpi_context,
            DataV2 {
                name: token_name.clone(),
                symbol: token_symbol.clone(),
                uri: token_uri.clone(),
                seller_fee_basis_points: 0,
                creators: None,
                collection: None,
                uses: None,
            },
            false, // Is mutable
            true,  // Update authority is signer
            None,  // Collection details
        )?;

        let signer_seeds: &[&[&[u8]]] = &[&[
            b"mint",
            token_name.as_bytes(),
            token_symbol.as_bytes(),
            token_uri.as_bytes(),
            &[ctx.bumps.mint_a],
        ]];
        // 2. mint `mint_a` tokens to vault
        let mint_cpi = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.mint_a.to_account_info(),
                to: ctx.accounts.vault_a.to_account_info(),
                authority: ctx.accounts.mint_a.to_account_info(),
            },
        ).with_signer(signer_seeds);

        mint_to(mint_cpi, mint_amount)?;
        Ok(())
    }

    // 1. transfer funding tokens to contract
    // 2. transfer `mint_a` tokens from vault to user
    pub fn redeem(ctx: Context<Redeem>, amount: u64) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(token_name: String, token_symbol: String, token_uri: String)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    // Create mint account
    #[account(
        init,
        seeds = [b"mint".as_ref(), token_name.as_bytes(), token_symbol.as_bytes(), token_uri.as_bytes()],
        bump,
        payer = payer,
        mint::decimals = 9,
        // tune this to your liking
        mint::authority = mint_a.key(),
        mint::freeze_authority = mint_a.key(),
    )]
    pub mint_a: Account<'info, Mint>,

    /// CHECK: This account is not initialized in this instruction
    #[account(
        mut,
        seeds = [b"metadata".as_ref(), token_metadata_program.key().as_ref(), mint_a.key().as_ref()],
        bump,
        seeds::program = token_metadata_program.key(),
    )]
    pub metadata_a: UncheckedAccount<'info>,

    #[account(
        init,
        payer = payer,
        associated_token::mint = mint_a,
        associated_token::authority = mint_a,
    )]
    // needs to mint tokens to this account to start
    pub vault_a: Account<'info, TokenAccount>,

    // funding mint for the token contract accepts
    pub mint_b_funding: Account<'info, Mint>,

    // system programs
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_metadata_program: Program<'info, Metadata>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct Redeem<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    pub mint_a: Account<'info, Mint>,
    pub mint_b: Account<'info, Mint>,
}

// MARK: - State

pub struct State {
    pub mint_a: Pubkey,
    pub mint_b: Pubkey,
    pub a_distributed_amount: u64,
}
