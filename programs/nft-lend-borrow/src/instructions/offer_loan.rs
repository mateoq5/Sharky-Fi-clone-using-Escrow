pub use anchor_lang::prelude::*;

use anchor_lang::system_program;
use anchor_spl::token::Token;

pub use crate::states::{CollectionPool, Offer};

#[derive(Accounts)]
pub struct OfferLoan<'info> {
    #[account(
        init,
        seeds=[
            b"offer",
            collection_pool.key().as_ref(),
            lender.key().as_ref(),
            collection_pool.total_offers.to_string().as_bytes(),
        ],
        bump,
        payer=lender,
        space=Offer::LEN
    )]
    pub offer_loan: Box<Account<'info, Offer>>,

    /// CHECK: This is safe
    #[account(
        init,
        seeds=[
            b"vault-token-account",
            collection_pool.key().as_ref(),
            lender.key().as_ref(),
            collection_pool.total_offers.to_string().as_bytes(),
        ],
        bump,
        payer = lender,
        space = 8
    )]
    pub vault_account: AccountInfo<'info>,

    #[account(mut)]
    pub collection_pool: Box<Account<'info, CollectionPool>>,

    #[account(mut)]
    pub lender: Signer<'info>,

    pub token_program: Program<'info, Token>,

    pub system_program: Program<'info, System>,
}

impl<'info> OfferLoan<'info> {
    fn transfer_to_vault_context(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, system_program::Transfer<'info>> {
        let cpi_accounts = system_program::Transfer {
            from: self.lender.to_account_info().clone(),
            to: self.vault_account.to_account_info().clone(),
        };

        CpiContext::new(self.system_program.to_account_info(), cpi_accounts)
    }

    // fn set_authority_context(&self) -> CpiContext<'_, '_, '_, 'info, SetAuthority<'info>> {
    //     let cpi_accounts = SetAuthority {
    //         account_or_mint: self.vault_account.to_account_info().clone(),
    //         current_authority: self.lender.to_account_info().clone(),
    //     };

    //     CpiContext::new(self.token_program.to_account_info().clone(), cpi_accounts)
    // }
}

pub fn handler(ctx: Context<OfferLoan>, offer_amount: u64) -> Result<()> {
    let offer_account = &mut ctx.accounts.offer_loan;
    let collection = &mut ctx.accounts.collection_pool;

    offer_account.collection = collection.key();
    offer_account.offer_lamport_amount = offer_amount;
    offer_account.repay_lamport_amount = offer_amount + offer_amount * 10 / 100;
    offer_account.lender = ctx.accounts.lender.key();
    offer_account.bump = *ctx.bumps.get("offer_loan").unwrap();

    collection.total_offers += 1;

    // let (vault_account_authority, _offer_account_bump) = Pubkey::find_program_address(
    //     &[
    //         b"vault-token-account",
    //         collection.key().as_ref(),
    //         ctx.accounts.lender.key().as_ref(),
    //         collection.total_offers.to_string().as_bytes(),
    //     ],
    //     ctx.program_id,
    // );

    // token::set_authority(
    //     ctx.accounts.set_authority_context(),
    //     AuthorityType::AccountOwner,
    //     Some(vault_account_authority),
    // )?;

    system_program::transfer(ctx.accounts.transfer_to_vault_context(), offer_amount)?;

    Ok(())
}
