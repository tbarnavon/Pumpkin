#[allow(clippy::wildcard_imports)]
use super::*;
use pumpkin_fabric::handshake::{FabricHandshake, Outgoing, Step};
use pumpkin_protocol::java::client::config::{CConfigPing, CPluginMessage};

impl PendingConnection {
    pub async fn handle_login_acknowledged(
        &mut self,
        server: &Server,
    ) -> Option<PacketHandlerResult> {
        debug!("Handling login acknowledgement");
        self.connection_state.store(ConnectionState::Config);

        // With mods installed, Fabric's handshake runs first; `continue_configuration` follows
        // once it is done (see `handle_fabric_step`).
        if let Some(mods) = pumpkin_registry_ext::installed()
            && server.advanced_config.modded.fabric_handshake
        {
            let handshake = FabricHandshake::new(mods.namespaces.clone(), Vec::new());
            let start = handshake.start();
            self.fabric = Some(handshake);
            self.send_fabric_packets(start).await;
            return None;
        }
        self.continue_configuration(server).await;
        None
    }

    /// The vanilla configuration: brand, server links, resource pack or known packs.
    pub async fn continue_configuration(&mut self, server: &Server) {
        self.send_packet_now(&server.get_branding()).await;

        if server.advanced_config.server_links.enabled {
            let mut links: Vec<Link> = Vec::new();

            let bug_report = &server.advanced_config.server_links.bug_report;
            if !bug_report.is_empty() {
                links.push(Link::new(Label::BuiltIn(LinkType::BugReport), bug_report));
            }

            let support = &server.advanced_config.server_links.support;
            if !support.is_empty() {
                links.push(Link::new(Label::BuiltIn(LinkType::Support), support));
            }

            let status = &server.advanced_config.server_links.status;
            if !status.is_empty() {
                links.push(Link::new(Label::BuiltIn(LinkType::Status), status));
            }

            let feedback = &server.advanced_config.server_links.feedback;
            if !feedback.is_empty() {
                links.push(Link::new(Label::BuiltIn(LinkType::Feedback), feedback));
            }

            let community = &server.advanced_config.server_links.community;
            if !community.is_empty() {
                links.push(Link::new(Label::BuiltIn(LinkType::Community), community));
            }

            let website = &server.advanced_config.server_links.website;
            if !website.is_empty() {
                links.push(Link::new(Label::BuiltIn(LinkType::Website), website));
            }

            let forums = &server.advanced_config.server_links.forums;
            if !forums.is_empty() {
                links.push(Link::new(Label::BuiltIn(LinkType::Forums), forums));
            }

            let news = &server.advanced_config.server_links.news;
            if !news.is_empty() {
                links.push(Link::new(Label::BuiltIn(LinkType::News), news));
            }

            let announcements = &server.advanced_config.server_links.announcements;
            if !announcements.is_empty() {
                links.push(Link::new(
                    Label::BuiltIn(LinkType::Announcements),
                    announcements,
                ));
            }

            for (key, value) in &server.advanced_config.server_links.custom {
                links.push(Link::new(
                    Label::TextComponent(TextComponent::text(key.clone()).into()),
                    value,
                ));
            }

            self.send_packet_now(&CConfigServerLinks::new(&links)).await;
        }

        let resource_config = &server.advanced_config.resource_pack.java;
        if resource_config.enabled {
            let uuid = Uuid::new_v3(&uuid::Uuid::NAMESPACE_DNS, resource_config.url.as_bytes());
            let resource_pack = CConfigAddResourcePack::new(
                &uuid,
                &resource_config.url,
                &resource_config.sha1,
                resource_config.force,
                if resource_config.prompt_message.is_empty() {
                    None
                } else {
                    Some(TextComponent::text(resource_config.prompt_message.clone()))
                },
            );

            self.send_packet_now(&resource_pack).await;
        } else {
            self.send_known_packs(server).await;
        }
        debug!("login acknowledged");
    }

    pub async fn send_fabric_packets(&mut self, packets: Vec<Outgoing>) {
        for packet in packets {
            match packet {
                Outgoing::Payload { channel, data } => {
                    self.send_packet_now(&CPluginMessage::new(channel, &data))
                        .await;
                }
                Outgoing::Ping(id) => self.send_packet_now(&CConfigPing::new(id)).await,
            }
        }
    }

    /// Acts on a handshake step. Returns `false` if the packet was not part of the handshake.
    pub async fn handle_fabric_step(&mut self, server: &Server, step: Step) -> bool {
        match step {
            Step::NotHandled => return false,
            Step::Wait => {}
            Step::Send(packets) => self.send_fabric_packets(packets).await,
            Step::Done(packets) => {
                self.send_fabric_packets(packets).await;
                self.continue_configuration(server).await;
            }
            Step::Disconnect(message) => self.kick(TextComponent::text(message)).await,
        }
        true
    }

    pub async fn send_known_packs(&mut self, server: &Server) {
        let features = server.get_enabled_features();
        self.send_packet_now(&CFeatureFlags::new(&features)).await;
        let version_str = CURRENT_MC_VERSION.to_string();
        let loaded_packs = server.datapack_manager.get_loaded_packs();
        let known_packs = server.get_known_packs(&version_str, &loaded_packs);
        self.send_packet_now(&CKnownPacks::new(&known_packs)).await;
    }
}
