use anchor_lang::prelude::*;
use anchor_lang::solana_program::rent::{
    DEFAULT_EXEMPTION_THRESHOLD, DEFAULT_LAMPORTS_PER_BYTE_YEAR
};
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface::{token_metadata_initialize, Mint, Token2022, TokenAccount, TokenMetadataInitialize, 
    TransferChecked, transfer_checked, MintTo, mint_to};
use anchor_lang::system_program::{transfer as system_transfer, Transfer as SystemTransfer};
use spl_token_metadata_interface::state::TokenMetadata;
use spl_type_length_value::variable_len_pack::VariableLenPack;

declare_id!("QJqxGatfoyyHgeuxNgur5EWETdXGzYx2hBkdtrbT2gZ");

#[program]
pub mod token_2022 {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        name: String,
        symbol: String,
        uri: String,
    ) -> Result<()> {
        let token_metadata = TokenMetadata {
            name: name.clone(),
            symbol: symbol.clone(),
            uri: uri.clone(),
            ..Default::default()
        };

        // Add 4 extra bytes for size of MetadataExtension (2 bytes for type, 2 bytes for length)
        let data_len = 4 + token_metadata.get_packed_len()?;
        // Calculate lamports required for the additional metadata
        let lamports =
            data_len as u64 * DEFAULT_LAMPORTS_PER_BYTE_YEAR * DEFAULT_EXEMPTION_THRESHOLD as u64;
        // transfer additional lamports to the mint account
        system_transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                SystemTransfer {
                    from: ctx.accounts.signer.to_account_info(),
                    to: ctx.accounts.mint.to_account_info(),
                }
            ),
            lamports 
        )?;
        let ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            TokenMetadataInitialize {
                token_program_id: ctx.accounts.token_program.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
                metadata: ctx.accounts.mint.to_account_info(),
                mint_authority: ctx.accounts.signer.to_account_info(),
                update_authority: ctx.accounts.signer.to_account_info(),
            }
        );
        token_metadata_initialize(ctx, name, symbol, uri)?;
        Ok(())
    }

    // MARK: - Mint Token
    pub fn mint_token(
        ctx: Context<MintToken>,
        amount: u64,
    ) -> Result<()> { 
        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.mint.to_account_info(),
                to: ctx.accounts.vault.to_account_info(),
                authority: ctx.accounts.signer.to_account_info(),
            }
        );
        mint_to(cpi_ctx, amount)?;
        Ok(())
    }

    // MARK: - Transfer Token
    pub fn transfer_token(
        ctx: Context<TransferToken>,
        amount: u64,
    ) -> Result<()> {
        msg!("to_authority: {}", ctx.accounts.to_authority.key().to_string());
        msg!("from_vault: {}", ctx.accounts.from_vault.key().to_string());
        msg!("mint: {}", ctx.accounts.mint.key().to_string());
        msg!("to: {}", ctx.accounts.to.key().to_string());

        let mint_key = ctx.accounts.mint.key();
        let signer_seeds: &[&[&[u8]]] = &[&[
            b"vault",
            mint_key.as_ref(),
            &[ctx.bumps.from_vault]
        ]];
        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.from_vault.to_account_info(),
                to: ctx.accounts.to.to_account_info(),
                authority: ctx.accounts.from_vault.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
            }
        ).with_signer(signer_seeds);

        transfer_checked(cpi_ctx, amount, ctx.accounts.mint.decimals)?;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        init,
        payer = signer,
        mint::decimals = 9,
        mint::authority = signer,
        extensions::metadata_pointer::authority = signer,
        extensions::metadata_pointer::metadata_address = mint,
    )]
    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        init,
        payer = signer,
        seeds = [b"vault", mint.key().as_ref()],
        bump,
        token::mint = mint,
        token::authority = vault,
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    // must be token-2022 for metadata extension
    pub token_program: Program<'info, Token2022>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct MintToken<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(mut)]
    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        seeds = [b"vault", mint.key().as_ref()],
        bump,
        token::mint = mint,
        token::authority = vault,
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Program<'info, Token2022>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct TransferToken<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        seeds = [b"vault", mint.key().as_ref()],
        bump,
        token::mint = mint,
        token::authority = from_vault,
    )]
    pub from_vault: InterfaceAccount<'info, TokenAccount>,

    pub to_authority: SystemAccount<'info>,
    #[account(
        // init here will throw error if account already exists
        // init will also require system program and associated_token program
        mut, 
        associated_token::mint = mint,
        associated_token::authority = to_authority,
        associated_token::token_program = token_program,
    )]
    pub to: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Program<'info, Token2022>,
    // CAUTION: If you don't specify the associated token program,
    // the `to` ATA will be different from the ATA created by the client
    pub associated_token_program: Program<'info, AssociatedToken>,
}
