//! Miscellaneous type definitions used in packets.

use bitfield_struct::bitfield;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use valence_nbt::Compound;

use crate::{BlockPos, Decode, Encode, Ident, Text, VarInt};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Encode, Decode)]
pub enum HandshakeNextState {
    #[tag = 1]
    Status,
    #[tag = 2]
    Login,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Encode, Decode)]
pub struct PublicKeyData<'a> {
    pub timestamp: u64,
    pub public_key: &'a [u8],
    pub signature: &'a [u8],
}

#[derive(Clone, Debug, Encode, Decode)]
pub struct MessageAcknowledgment<'a> {
    pub last_seen: Vec<MessageAcknowledgmentEntry<'a>>,
    pub last_received: Option<MessageAcknowledgmentEntry<'a>>,
}

#[derive(Copy, Clone, Debug, Encode, Decode)]
pub struct MessageAcknowledgmentEntry<'a> {
    pub profile_id: Uuid,
    pub signature: &'a [u8],
}

#[derive(Copy, Clone, Debug, Encode, Decode)]
pub struct CommandArgumentSignature<'a> {
    pub argument_name: &'a str,
    pub signature: &'a [u8],
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Encode, Decode)]
pub enum ChatMode {
    Enabled,
    CommandsOnly,
    Hidden,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default, Encode, Decode)]
pub enum MainHand {
    Left,
    #[default]
    Right,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Encode, Decode)]
pub enum ClickContainerMode {
    Click,
    ShiftClick,
    Hotbar,
    CreativeMiddleClick,
    DropKey,
    Drag,
    DoubleClick,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Encode, Decode)]
pub enum Hand {
    Main,
    Off,
}

#[derive(Copy, Clone, PartialEq, Debug, Encode, Decode)]
pub enum EntityInteraction {
    Interact(Hand),
    Attack,
    InteractAt { target: [f32; 3], hand: Hand },
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Encode, Decode)]
pub enum DiggingStatus {
    StartedDigging,
    CancelledDigging,
    FinishedDigging,
    DropItemStack,
    DropItem,
    UpdateHeldItemState,
    SwapItemInHand,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Encode, Decode)]
pub enum Action {
    StartSneaking,
    StopSneaking,
    LeaveBed,
    StartSprinting,
    StopSprinting,
    StartJumpWithHorse,
    StopJumpWithHorse,
    OpenHorseInventory,
    StartFlyingWithElytra,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Encode, Decode)]
pub enum RecipeBookId {
    Crafting,
    Furnace,
    BlastFurnace,
    Smoker,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Encode, Decode)]
pub enum CommandBlockMode {
    Sequence,
    Auto,
    Redstone,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Encode, Decode)]
pub enum StructureBlockAction {
    UpdateData,
    SaveStructure,
    LoadStructure,
    DetectSize,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Encode, Decode)]
pub enum StructureBlockMode {
    Save,
    Load,
    Corner,
    Data,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Encode, Decode)]
pub enum StructureBlockMirror {
    None,
    LeftRight,
    FrontBack,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Encode, Decode)]
pub enum StructureBlockRotation {
    None,
    Clockwise90,
    Clockwise180,
    Counterclockwise90,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Encode, Decode, Serialize, Deserialize)]
pub struct Property<S = String> {
    pub name: S,
    pub value: S,
    pub signature: Option<S>,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Encode, Decode)]
pub enum Animation {
    SwingMainArm,
    TakeDamage,
    LeaveBed,
    SwingOffhand,
    CriticalEffect,
    MagicCriticalEffect,
}

#[derive(Clone, PartialEq, Debug, Encode, Decode)]
pub enum BossBarAction {
    Add {
        title: Text,
        health: f32,
        color: BossBarColor,
        division: BossBarDivision,
        flags: BossBarFlags,
    },
    Remove,
    UpdateHealth(f32),
    UpdateTitle(Text),
    UpdateStyle(BossBarColor, BossBarDivision),
    UpdateFlags(BossBarFlags),
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Encode, Decode)]
pub enum BossBarColor {
    Pink,
    Blue,
    Red,
    Green,
    Yellow,
    Purple,
    White,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Encode, Decode)]
pub enum BossBarDivision {
    NoDivision,
    SixNotches,
    TenNotches,
    TwelveNotches,
    TwentyNotches,
}

#[bitfield(u8)]
#[derive(PartialEq, Eq, Encode, Decode)]
pub struct BossBarFlags {
    pub darken_sky: bool,
    pub dragon_bar: bool,
    pub create_fog: bool,
    #[bits(5)]
    _pad: u8,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Encode, Decode)]
pub enum Difficulty {
    Peaceful,
    Easy,
    Normal,
    Hard,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Encode, Decode)]
pub enum SoundCategory {
    Master,
    Music,
    Record,
    Weather,
    Block,
    Hostile,
    Neutral,
    Player,
    Ambient,
    Voice,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Encode, Decode)]
pub enum GameEventKind {
    NoRespawnBlockAvailable,
    EndRaining,
    BeginRaining,
    ChangeGameMode,
    WinGame,
    DemoEvent,
    ArrowHitPlayer,
    RainLevelChange,
    ThunderLevelChange,
    PlayPufferfishStingSound,
    PlayElderGuardianMobAppearance,
    EnableRespawnScreen,
}

#[derive(Clone, PartialEq, Debug, Encode, Decode)]
pub struct ChunkDataBlockEntity {
    pub packed_xz: i8,
    pub y: i16,
    // TODO: block entity kind?
    pub kind: VarInt,
    pub data: Compound,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default, Encode, Decode)]
pub enum GameMode {
    #[default]
    Survival,
    Creative,
    Adventure,
    Spectator,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Encode, Decode)]
pub struct GlobalPos<'a> {
    pub dimension_name: Ident<&'a str>,
    pub position: BlockPos,
}

#[derive(Clone, PartialEq, Debug, Encode, Decode)]
pub struct AttributeProperty<'a> {
    pub key: Ident<&'a str>,
    pub value: f64,
    pub modifiers: Vec<AttributeModifier>,
}

#[derive(Clone, PartialEq, Debug, Encode, Decode)]
pub struct AttributeModifier {
    pub uuid: Uuid,
    pub amount: f64,
    pub operation: u8,
}

#[bitfield(u8)]
#[derive(PartialEq, Eq, Encode, Decode)]
pub struct DisplayedSkinParts {
    pub cape: bool,
    pub jacket: bool,
    pub left_sleeve: bool,
    pub right_sleeve: bool,
    pub left_pants_leg: bool,
    pub right_pants_leg: bool,
    pub hat: bool,
    _pad: bool,
}

#[bitfield(u8)]
#[derive(PartialEq, Eq, Encode, Decode)]
pub struct PlayerInputFlags {
    pub jump: bool,
    pub unmount: bool,
    #[bits(6)]
    _pad: u8,
}

#[bitfield(u8)]
#[derive(PartialEq, Eq, Encode, Decode)]
pub struct CommandBlockFlags {
    pub track_output: bool,
    pub conditional: bool,
    pub automatic: bool,
    #[bits(5)]
    _pad: u8,
}

#[bitfield(u8)]
#[derive(PartialEq, Eq, Encode, Decode)]
pub struct StructureBlockFlags {
    pub ignore_entities: bool,
    pub show_air: bool,
    pub show_bounding_box: bool,
    #[bits(5)]
    _pad: u8,
}

#[bitfield(u8)]
#[derive(PartialEq, Eq, Encode, Decode)]
pub struct SyncPlayerPosLookFlags {
    pub x: bool,
    pub y: bool,
    pub z: bool,
    pub y_rot: bool,
    pub x_rot: bool,
    #[bits(3)]
    _pad: u8,
}

#[bitfield(u8)]
#[derive(PartialEq, Eq, Encode, Decode)]
pub struct PlayerAbilitiesFlags {
    pub invulnerable: bool,
    pub flying: bool,
    pub allow_flying: bool,
    pub instant_break: bool,
    #[bits(4)]
    _pad: u8,
}

#[derive(Clone, PartialEq, Eq, Debug, Encode, Decode)]
pub struct TagGroup<'a> {
    pub kind: Ident<&'a str>,
    pub tags: Vec<Tag<'a>>,
}

#[derive(Clone, PartialEq, Eq, Debug, Encode, Decode)]
pub struct Tag<'a> {
    pub name: Ident<&'a str>,
    pub entries: Vec<VarInt>,
}

#[derive(Copy, Clone, PartialEq, Debug, Encode, Decode)]
pub struct Statistic {
    pub category_id: VarInt,
    pub statistic_id: VarInt,
    pub value: VarInt,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Encode, Decode)]
pub enum WindowType {
    Generic9x1,
    Generic9x2,
    Generic9x3,
    Generic9x4,
    Generic9x5,
    Generic9x6,
    Generic3x3,
    Anvil,
    Beacon,
    BlastFurnace,
    BrewingStand,
    Crafting,
    Enchantment,
    Furnace,
    Grindstone,
    Hopper,
    Lectern,
    Loom,
    Merchant,
    ShulkerBox,
    Smithing,
    Smoker,
    Cartography,
    Stonecutter,
}
