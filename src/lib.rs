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

        require!(agent.agent_pubkey == ctx.accounts.user.key(), 
        CustomError::Unauthorized
        );

        require!(agent.is_active, 
            CustomError::AlreadyInactive
        );

        agent.is_active = false;

        Ok(())
    }

    pub fn perform_action(ctx: Context<PerformAction>) -> Result<()> {
        let agent = &mut ctx.accounts.agent;
    
        require!(
            agent.is_active,
            CustomError::AgentInactive
        );
    
        agent.reputation = agent.reputation.checked_add(1).unwrap();
    
        Ok(())
    }
    

    pub fn reactivate_agent(ctx: Context<ReactivateAgent>) -> Result<()> {
        let agent = &mut ctx.accounts.agent;
    
        // Authority invariant
        require!(
            agent.agent_pubkey == ctx.accounts.user.key(),
            CustomError::Unauthorized
        );
    
        // State transition invariant
        require!(
            !agent.is_active,
            CustomError::AlreadyActive
        );
    
        agent.is_active = true;
    
        Ok(())
    }
    
    pub fn penalize_agent(ctx: Context<PenalizeAgent>, penalty_amount: u64) -> Result<()> {
        let agent = &mut ctx.accounts.agent;
        let config = &ctx.accounts.config;
    
        // Authority invariant: only admin can penalize
        require!(
            config.admin_pubkey == ctx.accounts.admin.key(),
            CustomError::Unauthorized
        );
        
        // Invariant 1: agent must be active
        require!(
            agent.is_active,
            CustomError::AgentInactive
        );
    
        // Invariant 2: reputation must be > 0
        require!(
            agent.reputation > 0,
            CustomError::AlreadyZero
        );

        require!(
            penalty_amount > 0 && penalty_amount <= agent.reputation,
            CustomError::ChoosePenalty
        );        
    
        // Safe decrement
        agent.reputation = agent.reputation.checked_sub(penalty_amount).unwrap();
    
        Ok(())
    }
    
    pub fn initialize_config(ctx: Context<InitializeConfig>) -> Result<()> {
        let config = &mut ctx.accounts.config;
    
        // Set the admin at initialization time
        config.admin_pubkey = ctx.accounts.user.key();
    
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

    #[msg("Agent is already active")]
    AlreadyActive,

    #[msg("reputation is already 0")]
    AlreadyZero,

    #[msg("Penalty must be greater than 0 and less than or equal to reputation")]
    ChoosePenalty,
}

#[account]
pub struct Agent {
    pub agent_pubkey: Pubkey,
    pub is_active: bool,
    pub reputation: u64,
}

#[account]
pub struct GlobalConfig {
    pub admin_pubkey: Pubkey,
}

#[derive(Accounts)]
pub struct RegisterAgent<'info> {
    #[account(
        init,
        payer = user,
        seeds = [b"agent", user.key().as_ref()],
        bump,
        space = 8 + 32 + 1 + 8
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

#[derive(Accounts)]
pub struct ReactivateAgent<'info> {
    #[account(
        mut,
        seeds = [b"agent", user.key().as_ref()],
        bump
    )]

    pub agent: Account<'info, Agent>,
    pub user: Signer<'info>,
}

#[derive(Accounts)]
pub struct PenalizeAgent<'info> {
    #[account(
        mut,
        seeds = [b"agent", agent.agent_pubkey.as_ref()],
        bump
    )]
    pub agent: Account<'info, Agent>,

    #[account(
        seeds = [b"config"],
        bump
    )]
    pub config: Account<'info, GlobalConfig>,

    pub admin: Signer<'info>,
}


#[derive(Accounts)]
pub struct InitializeConfig<'info> {
    #[account(
        init,
        payer = user,
        seeds = [b"config"],
        bump,
        space = 8 + 32
    )]
    pub config: Account<'info, GlobalConfig>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub system_program: Program<'info, System>,
}
