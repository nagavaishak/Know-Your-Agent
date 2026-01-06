use anchor_lang::prelude::*;

declare_id!("11111111111111111111111111111111"); // replace later

#[program]
pub mod agent_registry {
    use super::*;

    pub fn register_agent(ctx: Context<RegisterAgent>) -> Result<()> {
        let agent = &mut ctx.accounts.agent;
    
        agent.agent_pubkey = ctx.accounts.user.key();
        agent.is_active = true;
    
        Ok(())
    }

    pub fn deactivate_agent(ctx: Context<DeactivateAgent>) -> Result<()>{
        let agent = &mut ctx.accounts.agent;

        require!(agent.agent_pubkey == ctx.account.user.key(), 
        CustomError::Unauthorized
        );

        require!(agent.is_active, 
            CustomError::AlreadyInactive
        );

        agent.is_active = false;

        Ok(())
    }

    pub fn perform_action(ctx: Context<PerformAction>) -> Result<()> {
        let agent = &ctx.accounts.agent;
    
        require!(
            agent.is_active,
            CustomError::AgentInactive
        );
    
        Ok(())
    }
    

}

#[error_code]
pub enum CustomError {
    #[msg("Agent is inactive")]
    AgentInactive,

    #[msg("Only the owner can modify the agent")]
    Unauthorized,

    #[msg("You cannot deactivate an already inactive agent")]
    AlreadyInactive,

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

#[derive(Accounts)]
pub struct DeactivateAgent<'info> {
    #[account(
        mut,
        seeds = [b"agent", user.key().as_ref()],
        bump
    )]

    pub agent: Account<'info, Agent>,
    pub user: Signer<'info>,
}

#[derive(Accounts)]
pub struct PerformAction<'info> {
    #[account(
        mut,
        seeds = [b"agent", user.key().as_ref()],
        bump
    )]

    pub agent: Account<'info, Agent>,
    pub user: Signer<'info>,
}