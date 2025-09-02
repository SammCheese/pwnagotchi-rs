use std::{any::Any, pin::Pin};

use tokio::sync::{mpsc, oneshot};

use crate::core::{
  agent::{Agent, RunningMode},
  models::net::{AccessPoint, Station},
  session::LastSession,
  ui::state::StateValue,
  utils::mode_to_str,
};

type BoxFutureUnit = Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

type BoxFutureAny = Pin<Box<dyn Future<Output = Box<dyn Any + Send>> + Send + 'static>>;

pub enum AgentCommand {
  Sync(Box<dyn FnOnce(&mut Agent) + Send>),
  Async(Box<dyn (FnOnce(&mut Agent) -> BoxFutureUnit) + Send>),
  GetAccessPointsByChannel { respond_to: oneshot::Sender<Vec<(u8, Vec<AccessPoint>)>> },
  SetMode { mode: RunningMode },
  SetView { key: String, value: StateValue },
  Start,
  Stop,
  Recon,
  SetChannel { channel: u8 },
  Associate { ap: Box<AccessPoint>, throttle: Option<f32> },
  Deauth { ap: Box<AccessPoint>, sta: Box<Station>, throttle: Option<f32> },
  ParseLastSession { skip: Option<bool> },
}

#[derive(Clone)]

pub struct AgentHandle {
  pub command_tx: mpsc::Sender<AgentCommand>,
}

impl AgentHandle {
  pub async fn execute<F>(&self, f: F)
  where
    F: FnOnce(&mut Agent) + Send + 'static,
  {
    let _ = self.command_tx.send(AgentCommand::Sync(Box::new(f))).await;
  }

  pub async fn execute_async<F, Fut>(&self, f: F)
  where
    F: FnOnce(&mut Agent) -> Fut + Send + 'static,
    Fut: Future<Output = ()> + Send + 'static,
  {
    let _ = self
      .command_tx
      .send(AgentCommand::Async(Box::new(move |agent| Box::pin(f(agent)))))
      .await;
  }

  pub async fn send_command(&self, cmd: AgentCommand) {
    let _ = self.command_tx.send(cmd).await;
  }
}

pub fn spawn_agent(agent: Agent) -> AgentHandle {
  let (tx, mut rx) = mpsc::channel::<AgentCommand>(100);

  tokio::spawn(async move {
    let mut agent = agent;

    while let Some(cmd) = rx.recv().await {
      match cmd {
        AgentCommand::Sync(f) => {
          f(&mut agent);
        }
        AgentCommand::Async(f) => {
          f(&mut agent).await;
        }
        AgentCommand::GetAccessPointsByChannel { respond_to } => {
          let out = agent.get_access_points_by_channel().await;

          let _ = respond_to.send(out);
        }
        AgentCommand::SetMode { mode } => {
          agent.mode = mode;

          agent.automata.view.set("mode", StateValue::Text(mode_to_str(mode)));
        }
        AgentCommand::SetView { key, value } => {
          agent.automata.view.set(&key, value);
        }
        AgentCommand::Start => {
          agent.start().await;
        }
        AgentCommand::Stop => {
          agent.stop().await;
        }
        AgentCommand::Recon => {
          agent.recon().await;
        }
        AgentCommand::SetChannel { channel } => {
          agent.set_channel(channel).await;
        }
        AgentCommand::Associate { ap, throttle } => {
          agent.associate(&ap, throttle).await;
        }
        AgentCommand::Deauth { ap, sta, throttle } => {
          agent.deauth(&ap, &sta, throttle).await;
        }
        AgentCommand::ParseLastSession { skip } => {
          LastSession::new().parse(&agent.automata.view, skip);
        }
      }
    }
  });

  AgentHandle { command_tx: tx }
}
