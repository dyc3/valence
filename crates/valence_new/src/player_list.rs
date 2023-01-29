use std::borrow::Cow;
use std::collections::hash_map::{Entry as MapEntry, OccupiedEntry as OccupiedMapEntry};
use std::collections::HashMap;
use std::iter::FusedIterator;
use std::mem;

use bevy_ecs::prelude::*;
use tracing::warn;
use uuid::Uuid;
use valence_protocol::packets::s2c::play::{PlayerInfoRemove, SetTabListHeaderAndFooter};
use valence_protocol::packets::s2c::player_info_update::{
    Actions, Entry as PlayerInfoEntry, PlayerInfoUpdate,
};
use valence_protocol::types::{GameMode, Property};
use valence_protocol::Text;

use crate::client::Client;
use crate::packet::{PacketWriter, WritePacket};
use crate::server::Server;

/// The global list of players on a server visible by pressing the tab key by
/// default.
///
/// Each entry in the player list is intended to represent a connected client to
/// the server. In addition to a list of players, the player list has a header
/// and a footer which can contain arbitrary text.
#[derive(Debug, Resource)]
pub struct PlayerList {
    cached_update_packets: Vec<u8>,
    entries: HashMap<Uuid, Option<PlayerListEntry>>,
    header: Text,
    footer: Text,
    modified_header_or_footer: bool,
}

impl PlayerList {
    pub(crate) fn new() -> Self {
        Self {
            cached_update_packets: vec![],
            entries: HashMap::new(),
            header: Text::default(),
            footer: Text::default(),
            modified_header_or_footer: false,
        }
    }

    pub fn get(&self, uuid: Uuid) -> Option<&PlayerListEntry> {
        self.entries.get(&uuid).and_then(|opt| opt.as_ref())
    }

    pub fn get_mut(&mut self, uuid: Uuid) -> Option<&mut PlayerListEntry> {
        self.entries.get_mut(&uuid).and_then(|opt| opt.as_mut())
    }

    pub fn iter(&self) -> impl FusedIterator<Item = (Uuid, &PlayerListEntry)> + Clone + '_ {
        self.entries
            .iter()
            .filter_map(|(&uuid, opt)| opt.as_ref().map(|entry| (uuid, entry)))
    }

    pub fn iter_mut(&mut self) -> impl FusedIterator<Item = (Uuid, &mut PlayerListEntry)> + '_ {
        self.entries
            .iter_mut()
            .filter_map(|(&uuid, opt)| opt.as_mut().map(|entry| (uuid, entry)))
    }

    pub fn insert(&mut self, uuid: Uuid, entry: PlayerListEntry) -> Option<PlayerListEntry> {
        match self.entry(uuid) {
            Entry::Occupied(mut oe) => Some(oe.insert(entry)),
            Entry::Vacant(ve) => {
                ve.insert(entry);
                None
            }
        }
    }

    pub fn remove(&mut self, uuid: Uuid) -> Option<PlayerListEntry> {
        match self.entry(uuid) {
            Entry::Occupied(oe) => Some(oe.remove()),
            Entry::Vacant(_) => None,
        }
    }

    pub fn entry(&mut self, uuid: Uuid) -> Entry {
        match self.entries.entry(uuid) {
            MapEntry::Occupied(oe) if oe.get().is_some() => {
                Entry::Occupied(OccupiedEntry { entry: oe })
            }
            MapEntry::Occupied(oe) => Entry::Vacant(VacantEntry {
                entry: MapEntry::Occupied(oe),
            }),
            MapEntry::Vacant(ve) => Entry::Vacant(VacantEntry {
                entry: MapEntry::Vacant(ve),
            }),
        }
    }

    pub fn header(&self) -> &Text {
        &self.header
    }

    pub fn set_header(&mut self, header: impl Into<Text>) -> Text {
        let header = header.into();

        if header != self.header {
            self.modified_header_or_footer = true;
        }

        mem::replace(&mut self.header, header)
    }

    pub fn footer(&self) -> &Text {
        &self.footer
    }

    pub fn set_footer(&mut self, footer: impl Into<Text>) -> Text {
        let footer = footer.into();

        if footer != self.footer {
            self.modified_header_or_footer = true;
        }

        mem::replace(&mut self.footer, footer)
    }

    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(Uuid, &mut PlayerListEntry) -> bool,
    {
        self.entries.retain(|&uuid, opt| {
            if let Some(entry) = opt {
                if !f(uuid, entry) {
                    *opt = None;
                }
            }

            true
        });
    }

    pub fn clear(&mut self) {
        self.entries.values_mut().for_each(|e| *e = None);
    }

    pub(crate) fn write_init_packets(&self, mut writer: impl WritePacket) {
        let actions = Actions::new()
            .with_add_player(true)
            .with_update_game_mode(true)
            .with_update_listed(true)
            .with_update_latency(true)
            .with_update_display_name(true);

        let entries: Vec<_> = self
            .entries
            .iter()
            .filter_map(|(&uuid, opt)| {
                opt.as_ref().map(|entry| PlayerInfoEntry {
                    player_uuid: uuid,
                    username: &entry.username,
                    properties: entry.properties().into(),
                    chat_data: None,
                    listed: entry.listed,
                    ping: entry.ping,
                    game_mode: entry.game_mode,
                    display_name: entry.display_name.clone(),
                })
            })
            .collect();

        if !entries.is_empty() {
            writer.write_packet(&PlayerInfoUpdate {
                actions,
                entries: entries.into(),
            });
        }

        if !self.header.is_empty() || !self.footer.is_empty() {
            writer.write_packet(&SetTabListHeaderAndFooter {
                header: self.header.clone(),
                footer: self.footer.clone(),
            });
        }
    }
}

/// Represents a player entry in the [`PlayerList`].
#[derive(Clone, Debug)]
pub struct PlayerListEntry {
    username: String, // TODO: Username<String>?
    properties: Vec<Property>,
    game_mode: GameMode,
    old_game_mode: GameMode,
    ping: i32,
    display_name: Option<Text>,
    listed: bool,
    old_listed: bool,
    is_new: bool,
    modified_ping: bool,
    modified_display_name: bool,
}

impl Default for PlayerListEntry {
    fn default() -> Self {
        Self {
            username: String::new(),
            properties: vec![],
            game_mode: GameMode::default(),
            old_game_mode: GameMode::default(),
            ping: -1, // Negative indicates absence.
            display_name: None,
            old_listed: true,
            listed: true,
            is_new: true,
            modified_ping: false,
            modified_display_name: false,
        }
    }
}

impl PlayerListEntry {
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn with_username(mut self, username: impl Into<String>) -> Self {
        self.username = username.into();

        if self.username.chars().count() > 16 {
            warn!("player list username is longer than 16 characters");
        }

        self
    }

    #[must_use]
    pub fn with_properties(mut self, properties: impl Into<Vec<Property>>) -> Self {
        self.properties = properties.into();
        self
    }

    #[must_use]
    pub fn with_game_mode(mut self, game_mode: GameMode) -> Self {
        self.game_mode = game_mode;
        self
    }

    #[must_use]
    pub fn with_ping(mut self, ping: i32) -> Self {
        self.ping = ping;
        self
    }

    #[must_use]
    pub fn with_display_name(mut self, display_name: Option<impl Into<Text>>) -> Self {
        self.display_name = display_name.map(Into::into);
        self
    }

    #[must_use]
    pub fn with_listed(mut self, listed: bool) -> Self {
        self.listed = listed;
        self
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn properties(&self) -> &[Property] {
        &self.properties
    }

    pub fn game_mode(&self) -> GameMode {
        self.game_mode
    }

    pub fn set_game_mode(&mut self, game_mode: GameMode) {
        self.game_mode = game_mode;
    }

    pub fn ping(&self) -> i32 {
        self.ping
    }

    pub fn set_ping(&mut self, ping: i32) {
        if self.ping != ping {
            self.ping = ping;
            self.modified_ping = true;
        }
    }

    pub fn display_name(&self) -> Option<&Text> {
        self.display_name.as_ref()
    }

    pub fn set_display_name(&mut self, display_name: Option<impl Into<Text>>) -> Option<Text> {
        let display_name = display_name.map(Into::into);

        if self.display_name != display_name {
            self.modified_display_name = true;
        }

        mem::replace(&mut self.display_name, display_name)
    }

    pub fn is_listed(&self) -> bool {
        self.listed
    }

    pub fn set_listed(&mut self, listed: bool) {
        self.listed = listed;
    }

    fn clear_trackers(&mut self) {
        self.old_game_mode = self.game_mode;
        self.old_listed = self.listed;
        self.modified_ping = false;
        self.modified_display_name = false;
    }
}

#[derive(Debug)]
pub enum Entry<'a> {
    Occupied(OccupiedEntry<'a>),
    Vacant(VacantEntry<'a>),
}

#[derive(Debug)]
pub struct OccupiedEntry<'a> {
    entry: OccupiedMapEntry<'a, Uuid, Option<PlayerListEntry>>,
}

impl<'a> OccupiedEntry<'a> {
    pub fn key(&self) -> &Uuid {
        self.entry.key()
    }

    pub fn remove_entry(mut self) -> (Uuid, PlayerListEntry) {
        let mut entry = self.entry.get_mut().take().unwrap();
        let uuid = *self.entry.key();

        entry.is_new = false;

        (uuid, entry)
    }

    pub fn get(&self) -> &PlayerListEntry {
        self.entry.get().as_ref().unwrap()
    }

    pub fn get_mut(&mut self) -> &mut PlayerListEntry {
        self.entry.get_mut().as_mut().unwrap()
    }

    pub fn into_mut(self) -> &'a mut PlayerListEntry {
        self.entry.into_mut().as_mut().unwrap()
    }

    pub fn insert(&mut self, mut entry: PlayerListEntry) -> PlayerListEntry {
        let old_entry = self.get_mut();

        // Need to overwrite the entry if the username or properties changed because the
        // player list update packet doesn't support modifying these. Otherwise we can
        // just modify the existing entry.
        if old_entry.username != entry.username || old_entry.properties != entry.properties {
            entry.clear_trackers();
            entry.is_new = true;
            self.entry.insert(Some(entry)).unwrap()
        } else {
            PlayerListEntry::new()
                .with_game_mode(old_entry.game_mode)
                .with_ping(old_entry.ping)
                .with_display_name(old_entry.set_display_name(entry.display_name))
                .with_listed(old_entry.listed)
        }
    }

    pub fn remove(self) -> PlayerListEntry {
        self.remove_entry().1
    }
}

#[derive(Debug)]
pub struct VacantEntry<'a> {
    entry: MapEntry<'a, Uuid, Option<PlayerListEntry>>,
}

impl<'a> VacantEntry<'a> {
    pub fn key(&self) -> &Uuid {
        self.entry.key()
    }

    pub fn into_key(self) -> Uuid {
        *self.entry.key()
    }

    pub fn insert(self, mut entry: PlayerListEntry) -> &'a mut PlayerListEntry {
        entry.clear_trackers();
        entry.is_new = true;

        match self.entry {
            MapEntry::Occupied(mut oe) => {
                oe.insert(Some(entry));
                oe.into_mut().as_mut().unwrap()
            }
            MapEntry::Vacant(ve) => ve.insert(Some(entry)).as_mut().unwrap(),
        }
    }
}

pub(crate) fn update_player_list(
    player_list: ResMut<PlayerList>,
    server: Res<Server>,
    mut clients: Query<&mut Client>,
) {
    let pl = player_list.into_inner();

    let mut scratch = vec![];
    pl.cached_update_packets.clear();

    let mut writer = PacketWriter::new(
        &mut pl.cached_update_packets,
        server.compression_threshold(),
        &mut scratch,
    );

    let mut removed = vec![];

    pl.entries.retain(|&uuid, entry| {
        let Some(entry) = entry else {
            removed.push(uuid);
            return false
        };

        if entry.is_new {
            entry.is_new = false;

            // Send packets to initialize this entry.

            let mut actions = Actions::new().with_add_player(true);

            // We don't need to send data for fields if they have the default values.

            if entry.listed {
                actions.set_update_listed(true);
            }

            // Negative ping indicates absence.
            if entry.ping != 0 {
                actions.set_update_latency(true);
            }

            if entry.game_mode != GameMode::default() {
                actions.set_update_game_mode(true);
            }

            if entry.display_name.is_some() {
                actions.set_update_display_name(true);
            }

            entry.clear_trackers();

            let packet_entry = PlayerInfoEntry {
                player_uuid: uuid,
                username: &entry.username,
                properties: Cow::Borrowed(&entry.properties),
                chat_data: None,
                listed: entry.listed,
                ping: entry.ping,
                game_mode: entry.game_mode,
                display_name: entry.display_name.clone(),
            };

            writer.write_packet(&PlayerInfoUpdate {
                actions,
                entries: Cow::Borrowed(&[packet_entry]),
            });
        } else {
            let mut actions = Actions::new();

            if entry.game_mode != entry.old_game_mode {
                entry.old_game_mode = entry.game_mode;
                actions.set_update_game_mode(true);
            }

            if entry.listed != entry.old_listed {
                entry.old_listed = entry.listed;
                actions.set_update_listed(true);
            }

            if entry.modified_ping {
                entry.modified_ping = false;
                actions.set_update_latency(true);
            }

            if entry.modified_display_name {
                entry.modified_display_name = false;
                actions.set_update_display_name(true);
            }

            if u8::from(actions) != 0 {
                writer.write_packet(&PlayerInfoUpdate {
                    actions,
                    entries: Cow::Borrowed(&[PlayerInfoEntry {
                        player_uuid: uuid,
                        username: &entry.username,
                        properties: Cow::default(),
                        chat_data: None,
                        listed: entry.listed,
                        ping: entry.ping,
                        game_mode: entry.game_mode,
                        display_name: entry.display_name.clone(),
                    }]),
                });
            }
        }

        true
    });

    if !removed.is_empty() {
        writer.write_packet(&PlayerInfoRemove {
            uuids: removed.into(),
        });
    }

    if pl.modified_header_or_footer {
        pl.modified_header_or_footer = false;

        writer.write_packet(&SetTabListHeaderAndFooter {
            header: pl.header.clone(),
            footer: pl.footer.clone(),
        });
    }

    for mut client in &mut clients {
        if client.is_new() {
            let _ = pl.write_init_packets(client.packet_writer_mut());
        } else {
            client.write_packet_bytes(&pl.cached_update_packets);
        }
    }
}

/// A system which adds new clients to the player list.
pub fn add_new_clients_to_player_list(
    clients: Query<&Client, Added<Client>>,
    mut player_list: ResMut<PlayerList>,
) {
    for client in &clients {
        let entry = PlayerListEntry::new()
            .with_username(client.username())
            .with_properties(client.properties())
            .with_game_mode(client.game_mode())
            .with_ping(-1); // TODO

        player_list.insert(client.uuid(), entry);
    }
}

pub fn remove_disconnected_clients_from_player_list(
    clients: Query<&mut Client>,
    mut player_list: ResMut<PlayerList>,
) {
    for client in &clients {
        if client.is_disconnected() {
            player_list.remove(client.uuid());
        }
    }
}
