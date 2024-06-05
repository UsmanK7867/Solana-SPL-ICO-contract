use anchor_lang::solana_program::entrypoint::ProgramResult;
use anchor_lang::{prelude::*, solana_program::clock::Clock};
use anchor_spl::token::{self, Mint, Token, TokenAccount};
mod constants;
use crate::{constants::*};
// use std::str::FromStr;

// declare_id!("DzeBXeBmomdGZvaUFN9ELJvgvowfDE59sHKA7S3rzT3H");
declare_id!("DzeBXeBmomdGZvaUFN9ELJvgvowfDE59sHKA7S3rzT3H");

#[program]
pub mod ico {
    pub const ICO_MINT_ADDRESS: &str = "49Jx7fP8rFRac8KSeTRYhAzyugFsiuhFTi1gbzY9Dqoh";
    // pub const ICO_MINT_ADDRESS: &str = "FBKhAghAqzttng8UAAf7VuX7msiNAtVxgEsY4PrfZxP4";
    use super::*;

    /* 
    ===========================================================
        create_ico function use CreateIco struct
    ===========================================================
*/
    pub fn create_ico(
        ctx: Context<CreateIco>,
        phase_one_tokens: u64,
        phase_one_price: u64,
        phase_one_time: u64,
        phase_two_tokens: u64,
        phase_two_price: u64,
        phase_two_time: u64,
    ) -> Result<()> {
        msg!("create program ATA for hold ICO");
        // // transfer ICO admin to program ata
        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.ico_ata_for_admin.to_account_info(),
                to: ctx.accounts.ico_ata_for_ico_program.to_account_info(),
                authority: ctx.accounts.admin.to_account_info(),
            },
        );
        let ico_amount = phase_one_tokens + phase_two_tokens;
        token::transfer(cpi_ctx, ico_amount)?;
        msg!("send {} ICO to program ATA.", ico_amount);

        // save data in data PDA
        let clock = Clock::get()?;
        let data = &mut ctx.accounts.data;
        data.phase_one_price = phase_one_price;
        data.phase_one_tokens = phase_one_tokens;
        data.phase_one_time = (clock.unix_timestamp + phase_one_time as i64) as u64;
        data.phase_two_trice = phase_two_price;
        data.phase_two_tokens = phase_two_tokens;
        data.phase_two_time = (clock.unix_timestamp + phase_two_time as i64) as u64;
        data.admin = *ctx.accounts.admin.key;
        msg!("save data in program PDA.");
        Ok(())
    }

    /* 
    ===========================================================
        deposit_in_ico function use DepositInIco struct
    ===========================================================
*/
    pub fn deposit_in_ico(ctx: Context<DepositInIco>, ico_amount: u64) -> ProgramResult {
        if ctx.accounts.data.admin != *ctx.accounts.admin.key {
            return Err(ProgramError::IncorrectProgramId);
        }
        // transfer ICO admin to program ata
        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.ico_ata_for_admin.to_account_info(),
                to: ctx.accounts.ico_ata_for_ico_program.to_account_info(),
                authority: ctx.accounts.admin.to_account_info(),
            },
        );
        token::transfer(cpi_ctx, ico_amount)?;
        msg!("deposit {} ICO in program ATA.", ico_amount);
        Ok(())
    }

    /* 
    ===========================================================
        buy function use Buy struct
    ===========================================================
*/
    pub fn buy(
        ctx: Context<Buy>,
        _ico_ata_for_ico_program_bump: u8,
        sol_amount: u64,
        phase: u8,
    ) -> Result<()> {
        let data = &mut ctx.accounts.data;
        let current_time = Clock::get()?;
        if phase == 1 {
            (current_time.unix_timestamp as u64) < data.phase_one_time
        } else if phase == 2 {
            (current_time.unix_timestamp as u64) < data.phase_two_time
        } else {
            return Err(ProgramError::InvalidArgument.into());
        };
        // transfer sol from user to admin
        let ix = anchor_lang::solana_program::system_instruction::transfer(
            &ctx.accounts.user.key(),
            &ctx.accounts.admin.key(),
            sol_amount,
        );
        anchor_lang::solana_program::program::invoke(
            &ix,
            &[
                ctx.accounts.user.to_account_info(),
                ctx.accounts.admin.to_account_info(),
            ],
        )?;
        msg!("transfer {} sol to admin.", sol_amount);

        // transfer ICO from program to user ATA
        let ico_amount;
        if phase == 1 {
            ico_amount = (sol_amount * data.phase_one_price) / 1_000_000_000;
        } else {
            ico_amount = (sol_amount * data.phase_two_trice) / 1_000_000_000;
        };
        // let ico_mint_address = ctx.accounts.ico_mint.key();
        let seeds = &["ico5".as_bytes(), &[_ico_ata_for_ico_program_bump]];
        let signer = [&seeds[..]];
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.ico_ata_for_ico_program.to_account_info(),
                to: ctx.accounts.ico_ata_for_user.to_account_info(),
                authority: ctx.accounts.ico_ata_for_ico_program.to_account_info(),
            },
            &signer,
        );
        token::transfer(cpi_ctx, ico_amount)?;
        if phase == 1 {
            data.phase_one_tokens -= ico_amount;
            data.phase_one_sold_tokens += ico_amount;
            data.phase_one_sol += sol_amount;
        } else {
            data.phase_two_tokens -= ico_amount;
            data.phase_two_sold_tokens += ico_amount;
            data.phase_two_sol += sol_amount;
        }
        msg!("transfer {} ico to buyer/user.", ico_amount);
        Ok(())
    }

    /* 
    ===========================================================
        update_data function use UpdateData struct
    ===========================================================
*/
    pub fn update_data(ctx: Context<UpdateData>, phase: u8, new_price: u64) -> ProgramResult {
        if ctx.accounts.data.admin != *ctx.accounts.admin.key {
            return Err(ProgramError::IncorrectProgramId);
        }
        let data = &mut ctx.accounts.data;
        if phase == 1 {
            data.phase_one_price = new_price;
        } else if phase == 2 {
            data.phase_two_trice = new_price;
        } else {
            return Err(ProgramError::InvalidArgument.into());
        };
        msg!("update SOL/ICO {} ", new_price);
        Ok(())
    }

    /* 
    -----------------------------------------------------------
        CreateIco struct for create_ico function
    -----------------------------------------------------------
*/
    #[derive(Accounts)]
    pub struct CreateIco<'info> {
        // 1. PDA (pubkey) for ico ATA for our program.
        // seeds: [ico_mint + current program id] => "HashMap[seeds+bump] = pda"
        // token::mint: Token Program wants to know what kind of token this ATA is for
        // token::authority: It's a PDA so the authority is itself!
        #[account(
        init_if_needed,
        payer = admin,
        seeds = [ICO_SEED.as_bytes()],
        bump,
        token::mint = ico_mint,
        token::authority = ico_ata_for_ico_program,
    )]
        pub ico_ata_for_ico_program: Account<'info, TokenAccount>,

        #[account(init_if_needed, payer=admin, space=600, seeds=[DATA_SEED.as_bytes()], bump)]
        pub data: Account<'info, Data>,

        #[account(
        address = ICO_MINT_ADDRESS.parse::<Pubkey>().unwrap(),
    )]
        pub ico_mint: Account<'info, Mint>,

        #[account(mut)]
        pub ico_ata_for_admin: Account<'info, TokenAccount>,

        #[account(mut)]
        pub admin: Signer<'info>,

        pub system_program: Program<'info, System>,
        pub token_program: Program<'info, Token>,
        pub rent: Sysvar<'info, Rent>,
    }

    /* 
    -----------------------------------------------------------
        DepositInIco struct for deposit_in_ico function
    -----------------------------------------------------------
*/
    #[derive(Accounts)]
    pub struct DepositInIco<'info> {
        #[account(
        mut,
        seeds = [ICO_SEED.as_bytes()],
        bump,
    )]
        pub ico_ata_for_ico_program: Account<'info, TokenAccount>,

        #[account(
        mut,
        seeds = [DATA_SEED.as_bytes()],
        bump,
    )]
        pub data: Account<'info, Data>,

        #[account(
        address = ICO_MINT_ADDRESS.parse::<Pubkey>().unwrap(),
    )]
        pub ico_mint: Account<'info, Mint>,

        #[account(mut)]
        pub ico_ata_for_admin: Account<'info, TokenAccount>,

        #[account(mut)]
        pub admin: Signer<'info>,
        pub token_program: Program<'info, Token>,
    }

    /* 
    -----------------------------------------------------------
        Buy struct for buy function
    -----------------------------------------------------------
*/
    #[derive(Accounts)]
    #[instruction(_ico_ata_for_ico_program_bump: u8)]
    pub struct Buy<'info> {
        #[account(
        mut,
        seeds = [ICO_SEED.as_bytes()],
        bump,
    )]
        pub ico_ata_for_ico_program: Account<'info, TokenAccount>,

        #[account(
        mut,
        seeds = [DATA_SEED.as_bytes()],
        bump,
    )]
        pub data: Account<'info, Data>,

        #[account(
        address = ICO_MINT_ADDRESS.parse::<Pubkey>().unwrap(),
    )]
        pub ico_mint: Account<'info, Mint>,

        #[account(mut)]
        pub ico_ata_for_user: Account<'info, TokenAccount>,

        #[account(mut)]
        pub user: Signer<'info>,

        /// CHECK:
        #[account(mut)]
        pub admin: AccountInfo<'info>,

        pub token_program: Program<'info, Token>,
        pub system_program: Program<'info, System>,
    }
    // -----------------------------------------------------------
    //     UpdateData struct for updating ICO structure
    // -----------------------------------------------------------
    #[derive(Accounts)]
    pub struct UpdateData<'info> {
        #[account(mut)]
        pub data: Account<'info, Data>,
        #[account(mut)]
        pub admin: Signer<'info>,
        pub system_program: Program<'info, System>,
    }

    /* 
    -----------------------------------------------------------
        ICO Data struct for PDA Account
    -----------------------------------------------------------
  */
    #[account]
    pub struct Data {
        pub phase_one_time: u64,
        pub phase_one_price: u64,
        pub phase_one_tokens: u64,
        pub phase_one_sold_tokens: u64,
        pub phase_one_sol: u64,
        pub phase_two_time: u64,
        pub phase_two_trice: u64,
        pub phase_two_tokens: u64,
        pub phase_two_sold_tokens: u64,
        pub phase_two_sol: u64,
        pub admin: Pubkey,
    }
}
