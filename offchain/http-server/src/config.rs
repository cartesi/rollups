// Copyright Cartesi Pte. Ltd.
//
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.
//
use clap::{
    value_parser, Arg, Command, CommandFactory, FromArgMatches, Parser,
};

#[derive(Debug, Parser)]
pub struct HttpServerConfig {
    pub(crate) port: u16,
}

impl HttpServerConfig {
    /// Returns the HTTP server config and the app's config after parsing
    /// it from the command line and/or environment variables.
    ///
    /// The parametric type `C` must be a struct that derives `Parser`.
    pub fn parse<C: CommandFactory + FromArgMatches>(
        service: &'static str,
    ) -> (HttpServerConfig, C) {
        let command = <C as CommandFactory>::command();
        let command = add_port_arg(command, service);

        let matches = command.get_matches();
        let http_server_config: HttpServerConfig =
            FromArgMatches::from_arg_matches(&matches).unwrap();
        let inner_config: C =
            FromArgMatches::from_arg_matches(&matches).unwrap();
        (http_server_config, inner_config)
    }
}

fn add_port_arg<S: ToString>(command: Command, service: S) -> Command {
    let service = service.to_string().to_uppercase();
    command.arg(
        Arg::new("port")
            .long("http-server-port")
            .env(format!("{}_HTTP_SERVER_PORT", service))
            .value_parser(value_parser!(u16))
            .default_value("8080"),
    )
}
