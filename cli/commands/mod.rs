// Copyright (C) 2019-2022 Aleo Systems Inc.
// This file is part of the Aleo library.

// The Aleo library is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// The Aleo library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with the Aleo library. If not, see <https://www.gnu.org/licenses/>.

mod deploy;
pub use deploy::*;

mod node;
pub use node::*;

mod pour;
pub use pour::*;

mod execute;
pub use execute::*;

mod update;
pub use update::*;

mod view;
pub use view::*;

use anyhow::Result;
use clap::Parser;

#[derive(Debug, Parser)]
#[clap(
    name = "slingshot",
    author = "The Aleo Team <hello@aleo.org>",
    about = "A lightweight CLI for deploying programs and executing transactions on a development node.",
    setting = clap::AppSettings::ColoredHelp
)]
pub struct CLI {
    /// Specify the verbosity [options: 0, 1, 2, 3]
    #[clap(default_value = "2", short, long)]
    pub verbosity: u8,
    /// Specify a subcommand.
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Debug, Parser)]
pub enum Command {
    #[clap(name = "deploy")]
    Deploy(Deploy),
    #[clap(subcommand)]
    Node(Node),
    #[clap(name = "pour")]
    Pour(Pour),
    #[clap(name = "execute")]
    Execute(Execute),
    #[clap(subcommand)]
    Update(Update),
    #[clap(subcommand)]
    View(View),
}

impl Command {
    /// Parses the command.
    pub fn parse(self) -> Result<String> {
        match self {
            Self::Deploy(command) => command.parse(),
            Self::Node(command) => command.parse(),
            Self::Pour(command) => command.parse(),
            Self::Execute(command) => command.parse(),
            Self::Update(command) => command.parse(),
            Self::View(command) => command.parse(),
        }
    }
}
