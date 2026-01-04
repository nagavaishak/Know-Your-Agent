use anchor_lang::prelude::*;

declare_id!("11111111111111111111111111111111"); // replace later

#[program]
pub mod agent_registry {
    use super::*;

    pub fn register_agent(ctx: Context<RegisterAgent>) -> Result<()> {
        let agent = &mut ctx.accounts.agent;

        require!(agent.is_active, CustomError::AgentInactive);
        Ok(())
    }
}

pub enum CustomError{
    #[msg("Agent is inactive")]
    AgentInactive,
}

#[account]
pub struct Agent {
    pub agent_pubkey: Pubkey,
    pub is_active: bool,
}

#[derive(Accounts)]
pub struct RegisterAgent<'info> {
    #[account(
        init,
        payer = user,
        seeds = [b"agent", user.key().as_ref()],
        bump,
        space = 8 + 32 + 1
    )]
    pub agent: Account<'info, Agent>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub system_program: Program<'info, System>,
}
