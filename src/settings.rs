use asr::settings::{gui::Title, Gui};

/// Builds the `Settings` struct (one checkbox per story event and scene) plus
/// lookups by event id / scene name. Events are grouped under sub-headings.
///
/// Doc comments become the label; a second paragraph becomes the tooltip.
/// The field names are the saved setting keys, so don't rename them.
macro_rules! define_settings {
    (
        events: {
            $(
                $(#[doc = $group_doc:literal])*
                $group_field:ident {
                    $(
                        $(#[doc = $event_doc:literal])*
                        #[default = $event_default:expr]
                        $event_field:ident = $event_id:literal ($event_name:literal),
                    )*
                }
            )*
        }
        scenes: {
            $(
                $(#[doc = $scene_doc:literal])*
                #[default = $scene_default:expr]
                $scene_field:ident = $scene_id:literal,
            )*
        }
    ) => {
        #[derive(Gui)]
        pub struct Settings {
            /// Generic Settings
            _generic_settings: Title,

            /// Start on new game
            #[default = true]
            pub auto_start: bool,

            /// Reset on new game
            #[default = true]
            pub auto_reset: bool,

            /// Remove loading (real time minus transitions instead of in-game time)
            #[default = false]
            pub remove_loading: bool,

            /// Split the first time you save at each camp
            #[default = false]
            pub split_new_camp: bool,

            /// Split 25 seconds after the credits start
            #[default = false]
            pub split_end_credits: bool,

            /// Events
            _event_settings: Title,
            $(
                $(#[doc = $group_doc])*
                #[heading_level = 1]
                $group_field: Title,
                $(
                    $(#[doc = $event_doc])*
                    #[default = $event_default]
                    $event_field: bool,
                )*
            )*

            /// Scenes
            _scene_settings: Title,
            $(
                $(#[doc = $scene_doc])*
                #[default = $scene_default]
                $scene_field: bool,
            )*
        }

        impl Settings {
            pub fn get_event_enabled(&self, event_id: i32) -> bool {
                match event_id {
                    $($(
                        $event_id => self.$event_field,
                    )*)*
                    _ => false,
                }
            }

            pub fn get_scene_enabled(&self, scene_id: &str) -> bool {
                match scene_id {
                    $(
                        $scene_id => self.$scene_field,
                    )*
                    _ => false,
                }
            }

            /// The game's own name for a story event (`StoryEvent` enum member).
            pub fn get_event_name(event_id: i32) -> Option<&'static str> {
                match event_id {
                    $($(
                        $event_id => Some($event_name),
                    )*)*
                    _ => None,
                }
            }
        }
    };
}

define_settings! {
    events: {
        /// Story
        _events_story {
            /// Game started
            ///
            /// Story event Started (0x1)
            #[default = false]
            event_started = 0x1 ("Started"),

            /// Started credits
            ///
            /// Story event Ended (0x2)
            #[default = true]
            event_ended = 0x2 ("Ended"),

            /// Hit lever in Guardhouse
            ///
            /// Story event OpenedEldarDoor (0x3)
            #[default = false]
            event_opened_eldar_door = 0x3 ("OpenedEldarDoor"),

            /// Got first arrow of freedom
            ///
            /// Story event OpenedFirstWeaponChest (0x4)
            #[default = true]
            event_opened_first_weapon_chest = 0x4 ("OpenedFirstWeaponChest"),
        }

        /// Bosses
        _events_bosses {
            /// Final Boss Kill
            ///
            /// Story event FinalBoss_Kill (0x31)
            #[default = false]
            event_final_boss_kill = 0x31 ("FinalBoss_Kill"),

            /// Killed Augustus
            ///
            /// Story event Boss_Kill_1 (0x32)
            #[default = true]
            event_boss_kill_1 = 0x32 ("Boss_Kill_1"),

            /// Killed Belia
            ///
            /// Story event Boss_Kill_2 (0x33)
            #[default = false]
            event_boss_kill_2 = 0x33 ("Boss_Kill_2"),

            /// Boss Kill 3
            ///
            /// Story event Boss_Kill_3 (0x34)
            #[default = false]
            event_boss_kill_3 = 0x34 ("Boss_Kill_3"),

            /// Boss Kill 4
            ///
            /// Story event Boss_Kill_4 (0x35)
            #[default = false]
            event_boss_kill_4 = 0x35 ("Boss_Kill_4"),

            /// Boss Kill 5
            ///
            /// Story event Boss_Kill_5 (0x36)
            #[default = false]
            event_boss_kill_5 = 0x36 ("Boss_Kill_5"),

            /// Killed Eldar tv form
            ///
            /// Story event Boss_Kill_6 (0x37)
            #[default = true]
            event_boss_kill_6 = 0x37 ("Boss_Kill_6"),

            /// Boss Kill 7
            ///
            /// Story event Boss_Kill_7 (0x38)
            #[default = false]
            event_boss_kill_7 = 0x38 ("Boss_Kill_7"),
        }

        /// Minibosses
        _events_minibosses {
            /// Killed tank
            ///
            /// Story event Miniboss_Kill_1 (0x39)
            #[default = true]
            event_miniboss_kill_1 = 0x39 ("Miniboss_Kill_1"),

            /// Killed rock of remembrance guardian
            ///
            /// Story event Miniboss_Kill_2 (0x3A)
            #[default = false]
            event_miniboss_kill_2 = 0x3A ("Miniboss_Kill_2"),

            /// Killed stone of intention guardian
            ///
            /// Story event Miniboss_Kill_3 (0x3B)
            #[default = false]
            event_miniboss_kill_3 = 0x3B ("Miniboss_Kill_3"),

            /// Miniboss Kill 4
            ///
            /// Story event Miniboss_Kill_4 (0x3C)
            #[default = false]
            event_miniboss_kill_4 = 0x3C ("Miniboss_Kill_4"),
        }

        /// Power-ups
        _events_power_ups {
            /// Got stone of creation
            ///
            /// Story event PU_Stone_Creation (0x64)
            #[default = false]
            event_pu_stone_creation = 0x64 ("PU_Stone_Creation"),

            /// Got rock of Remembrance
            ///
            /// Story event PU_Stone_Remembrance (0x65)
            #[default = true]
            event_pu_stone_remembrance = 0x65 ("PU_Stone_Remembrance"),

            /// Got stone of Intention
            ///
            /// Story event PU_Stone_Intention (0x66)
            #[default = true]
            event_pu_stone_intention = 0x66 ("PU_Stone_Intention"),

            /// Got pearl of dreams
            ///
            /// Story event PU_Stone_Dreams (0x67)
            #[default = true]
            event_pu_stone_dreams = 0x67 ("PU_Stone_Dreams"),

            /// PU_Health
            ///
            /// Story event PU_Health (0x6E)
            #[default = false]
            event_pu_health = 0x6E ("PU_Health"),

            /// Got first weapon
            ///
            /// Story event PU_Ammo (0x6F)
            #[default = false]
            event_pu_ammo = 0x6F ("PU_Ammo"),

            /// Got first health flask
            ///
            /// Story event PU_HealthFlask (0x70)
            #[default = true]
            event_pu_health_flask = 0x70 ("PU_HealthFlask"),

            /// Got map
            ///
            /// Story event PU_Map (0x71)
            #[default = false]
            event_pu_map = 0x71 ("PU_Map"),

            /// PU_Shield
            ///
            /// Story event PU_Shield (0x72)
            #[default = false]
            event_pu_shield = 0x72 ("PU_Shield"),

            /// Got first mana flask
            ///
            /// Story event PU_ManaFlask (0x73)
            #[default = false]
            event_pu_mana_flask = 0x73 ("PU_ManaFlask"),

            /// Got first health flask upgrade
            ///
            /// Story event PU_HealthFlaskUpgrade (0x74)
            #[default = false]
            event_pu_health_flask_upgrade = 0x74 ("PU_HealthFlaskUpgrade"),

            /// Got first mana flask upgrade
            ///
            /// Story event PU_ManaFlaskUpgrade (0x75)
            #[default = false]
            event_pu_mana_flask_upgrade = 0x75 ("PU_ManaFlaskUpgrade"),

            /// PU_Money
            ///
            /// Story event PU_Money (0x76)
            #[default = false]
            event_pu_money = 0x76 ("PU_Money"),

            /// PU_DandaraArrow
            ///
            /// Story event PU_DandaraArrow (0x77)
            #[default = false]
            event_pu_dandara_arrow = 0x77 ("PU_DandaraArrow"),

            /// Got first mana weapon
            ///
            /// Story event PU_ManaWeapon (0x78)
            #[default = false]
            event_pu_mana_weapon = 0x78 ("PU_ManaWeapon"),

            /// PU_Money_Fear
            ///
            /// Story event PU_Money_Fear (0x79)
            #[default = false]
            event_pu_money_fear = 0x79 ("PU_Money_Fear"),

            /// PU_SuperDandara
            ///
            /// Story event PU_SuperDandara (0x7A)
            #[default = false]
            event_pu_super_dandara = 0x7A ("PU_SuperDandara"),

            /// PU_FearWeapon
            ///
            /// Story event PU_FearWeapon (0x7B)
            #[default = false]
            event_pu_fear_weapon = 0x7B ("PU_FearWeapon"),
        }

        /// Weapons
        _events_weapons {
            /// Got missile weapon
            ///
            /// Story event Weapon_Missile (0x96)
            #[default = true]
            event_weapon_missile = 0x96 ("Weapon_Missile"),

            /// Weapon_EnergyBall
            ///
            /// Story event Weapon_EnergyBall (0x97)
            #[default = false]
            event_weapon_energy_ball = 0x97 ("Weapon_EnergyBall"),

            /// Got memory shaft
            ///
            /// Story event Weapon_Remembrance (0x98)
            #[default = true]
            event_weapon_remembrance = 0x98 ("Weapon_Remembrance"),

            /// Weapon_Bounce
            ///
            /// Story event Weapon_Bounce (0x99)
            #[default = false]
            event_weapon_bounce = 0x99 ("Weapon_Bounce"),

            /// Weapon_Boomerang
            ///
            /// Story event Weapon_Boomerang (0x9A)
            #[default = false]
            event_weapon_boomerang = 0x9A ("Weapon_Boomerang"),

            /// Weapon_Vaccum
            ///
            /// Story event Weapon_Vaccum (0x9B)
            #[default = false]
            event_weapon_vaccum = 0x9B ("Weapon_Vaccum"),

            /// Weapon_WaterBomb
            ///
            /// Story event Weapon_WaterBomb (0x9C)
            #[default = false]
            event_weapon_water_bomb = 0x9C ("Weapon_WaterBomb"),

            /// Weapon_Teleport
            ///
            /// Story event Weapon_Teleport (0x9D)
            #[default = false]
            event_weapon_teleport = 0x9D ("Weapon_Teleport"),

            /// Weapon_Firewall
            ///
            /// Story event Weapon_Firewall (0x9E)
            #[default = false]
            event_weapon_firewall = 0x9E ("Weapon_Firewall"),
        }

        /// Dialogues
        _events_dialogues {
            /// Activated painter
            ///
            /// Story event D_Painter (0xC8)
            #[default = false]
            event_d_painter = 0xC8 ("D_Painter"),

            /// Activated musician
            ///
            /// Story event D_Musician (0xC9)
            #[default = false]
            event_d_musician = 0xC9 ("D_Musician"),

            /// Talked to Lazuli at bridge
            ///
            /// Story event D_CreatGuardBridge (0xCA)
            #[default = false]
            event_d_creat_guard_bridge = 0xCA ("D_CreatGuardBridge"),

            /// Talked to Eldar 1
            ///
            /// Story event D_Eldar1 (0xCB)
            #[default = false]
            event_d_eldar1 = 0xCB ("D_Eldar1"),
        }

        /// Characters
        _events_characters {
            /// Talked to painter
            ///
            /// Story event Char_Painter (0x12C)
            #[default = false]
            event_char_painter = 0x12C ("Char_Painter"),

            /// Talked to musician
            ///
            /// Story event Char_Musician (0x12D)
            #[default = false]
            event_char_musician = 0x12D ("Char_Musician"),

            /// Talked to Jhonny
            ///
            /// Story event Char_GameDev (0x12E)
            #[default = false]
            event_char_game_dev = 0x12E ("Char_GameDev"),

            /// Discovered first camp
            ///
            /// Story event Char_Boss1_NoName (0x12F)
            #[default = false]
            event_char_boss1_no_name = 0x12F ("Char_Boss1_NoName"),

            /// Started Augustus fight
            ///
            /// Story event Char_A1Boss (0x130)
            #[default = false]
            event_char_a1_boss = 0x130 ("Char_A1Boss"),

            /// Char_A2Boss
            ///
            /// Story event Char_A2Boss (0x131)
            #[default = false]
            event_char_a2_boss = 0x131 ("Char_A2Boss"),

            /// Char_Writer
            ///
            /// Story event Char_Writer (0x132)
            #[default = false]
            event_char_writer = 0x132 ("Char_Writer"),

            /// Talked to Eldar 1
            ///
            /// Story event Char_AFBoss1 (0x133)
            #[default = false]
            event_char_af_boss1 = 0x133 ("Char_AFBoss1"),

            /// Started Eldar 2
            ///
            /// Story event Char_AFBoss2 (0x134)
            #[default = false]
            event_char_af_boss2 = 0x134 ("Char_AFBoss2"),

            /// Char_HeartCourage
            ///
            /// Story event Char_HeartCourage (0x135)
            #[default = false]
            event_char_heart_courage = 0x135 ("Char_HeartCourage"),

            /// Talked to Lazuli at Stone of Creation
            ///
            /// Story event Char_Lazuli (0x13A)
            #[default = false]
            event_char_lazuli = 0x13A ("Char_Lazuli"),

            /// Char_Lazuli_NoName
            ///
            /// Story event Char_Lazuli_NoName (0x13B)
            #[default = false]
            event_char_lazuli_no_name = 0x13B ("Char_Lazuli_NoName"),
        }

        /// HUD
        _events_hud {
            /// HUD_Unlock
            ///
            /// Story event HUD_Unlock (0xFA)
            #[default = false]
            event_hud_unlock = 0xFA ("HUD_Unlock"),

            /// HUD_TutorialClear
            ///
            /// Story event HUD_TutorialClear (0xFB)
            #[default = false]
            event_hud_tutorial_clear = 0xFB ("HUD_TutorialClear"),
        }

        /// Cutscenes
        _events_cutscenes {
            /// Augustus door shake cutscene
            ///
            /// Story event Cutscene_FavelaHUB (0x1F4)
            #[default = false]
            event_cutscene_favela_hub = 0x1F4 ("Cutscene_FavelaHUB"),

            /// Cutscene_CampDiscovery
            ///
            /// Story event Cutscene_CampDiscovery (0x1F5)
            #[default = false]
            event_cutscene_camp_discovery = 0x1F5 ("Cutscene_CampDiscovery"),
        }

        /// Trials of Fear (DLC)
        _events_dlc {
            /// DLCF_EntranceAccessUnlocked
            ///
            /// Story event DLCF_EntranceAccessUnlocked (0x5)
            #[default = false]
            event_dlcf_entrance_access_unlocked = 0x5 ("DLCF_EntranceAccessUnlocked"),

            /// DLCF_FacedTheMasters
            ///
            /// Story event DLCF_FacedTheMasters (0x6)
            #[default = false]
            event_dlcf_faced_the_masters = 0x6 ("DLCF_FacedTheMasters"),

            /// DLCF_FreedNara
            ///
            /// Story event DLCF_FreedNara (0x7)
            #[default = false]
            event_dlcf_freed_nara = 0x7 ("DLCF_FreedNara"),

            /// DLCF_TimeFlagTrap
            ///
            /// Story event DLCF_TimeFlagTrap (0x8)
            #[default = false]
            event_dlcf_time_flag_trap = 0x8 ("DLCF_TimeFlagTrap"),

            /// DLCF_LazuliDeath
            ///
            /// Story event DLCF_LazuliDeath (0x9)
            #[default = false]
            event_dlcf_lazuli_death = 0x9 ("DLCF_LazuliDeath"),

            /// DLCF_FearEnded
            ///
            /// Story event DLCF_FearEnded (0xA)
            #[default = false]
            event_dlcf_fear_ended = 0xA ("DLCF_FearEnded"),

            /// PU_DLCF_FinalKey
            ///
            /// Story event PU_DLCF_FinalKey (0x6C)
            #[default = false]
            event_pu_dlcf_final_key = 0x6C ("PU_DLCF_FinalKey"),

            /// D_DLCF_MastersIntroduction
            ///
            /// Story event D_DLCF_MastersIntroduction (0xCC)
            #[default = false]
            event_d_dlcf_masters_introduction = 0xCC ("D_DLCF_MastersIntroduction"),

            /// D_DLCF_ExplorerFinish
            ///
            /// Story event D_DLCF_ExplorerFinish (0xCD)
            #[default = false]
            event_d_dlcf_explorer_finish = 0xCD ("D_DLCF_ExplorerFinish"),

            /// D_DLCF_NobleFinish
            ///
            /// Story event D_DLCF_NobleFinish (0xCE)
            #[default = false]
            event_d_dlcf_noble_finish = 0xCE ("D_DLCF_NobleFinish"),

            /// D_DLCF_PersistentFinish
            ///
            /// Story event D_DLCF_PersistentFinish (0xCF)
            #[default = false]
            event_d_dlcf_persistent_finish = 0xCF ("D_DLCF_PersistentFinish"),

            /// D_DLCF_ExplorerStart
            ///
            /// Story event D_DLCF_ExplorerStart (0xD0)
            #[default = false]
            event_d_dlcf_explorer_start = 0xD0 ("D_DLCF_ExplorerStart"),

            /// D_DLCF_NobleStart
            ///
            /// Story event D_DLCF_NobleStart (0xD1)
            #[default = false]
            event_d_dlcf_noble_start = 0xD1 ("D_DLCF_NobleStart"),

            /// D_DLCF_PersistentStart
            ///
            /// Story event D_DLCF_PersistentStart (0xD2)
            #[default = false]
            event_d_dlcf_persistent_start = 0xD2 ("D_DLCF_PersistentStart"),

            /// Char_DLCF_Boss
            ///
            /// Story event Char_DLCF_Boss (0x136)
            #[default = false]
            event_char_dlcf_boss = 0x136 ("Char_DLCF_Boss"),

            /// Char_DLCF_Explorer
            ///
            /// Story event Char_DLCF_Explorer (0x137)
            #[default = false]
            event_char_dlcf_explorer = 0x137 ("Char_DLCF_Explorer"),

            /// Char_DLCF_Noble
            ///
            /// Story event Char_DLCF_Noble (0x138)
            #[default = false]
            event_char_dlcf_noble = 0x138 ("Char_DLCF_Noble"),

            /// Char_DLCF_Persistent
            ///
            /// Story event Char_DLCF_Persistent (0x139)
            #[default = false]
            event_char_dlcf_persistent = 0x139 ("Char_DLCF_Persistent"),

            /// Char_DLCF_Nara
            ///
            /// Story event Char_DLCF_Nara (0x13C)
            #[default = false]
            event_char_dlcf_nara = 0x13C ("Char_DLCF_Nara"),

            /// Char_DLCF_ElderDandara
            ///
            /// Story event Char_DLCF_ElderDandara (0x13D)
            #[default = false]
            event_char_dlcf_elder_dandara = 0x13D ("Char_DLCF_ElderDandara"),
        }
    }
    scenes: {
        /// Crib of Creation
        ///
        /// Scene A1_GreatRuins
        #[default = false]
        scene_a1_great_ruins = "A1_GreatRuins",

        /// Creation Meadow
        ///
        /// Scene A1_ForestEdge
        #[default = false]
        scene_a1_forest_edge = "A1_ForestEdge",

        /// Side Gate
        ///
        /// Scene A1_CityEntrance
        #[default = false]
        scene_a1_city_entrance = "A1_CityEntrance",

        /// The Village Center
        ///
        /// Scene A1_FavelaHub
        #[default = false]
        scene_a1_favela_hub = "A1_FavelaHub",

        /// Paint Well
        ///
        /// Scene A1_PainterPath1
        #[default = false]
        scene_a1_painter_path1 = "A1_PainterPath1",

        /// Abandoned House
        ///
        /// Scene A1_PainterPath2
        #[default = false]
        scene_a1_painter_path2 = "A1_PainterPath2",

        /// Community Street
        ///
        /// Scene A1_PainterPath3
        #[default = false]
        scene_a1_painter_path3 = "A1_PainterPath3",

        /// The Rich and the Poor
        ///
        /// Scene A1_PainterPath4
        #[default = false]
        scene_a1_painter_path4 = "A1_PainterPath4",

        /// Tarsila's House
        ///
        /// Scene A1_PainterHouse
        #[default = false]
        scene_a1_painter_house = "A1_PainterHouse",

        /// Backyard
        ///
        /// Scene A1_PainterExit1
        #[default = false]
        scene_a1_painter_exit1 = "A1_PainterExit1",

        /// Sewers
        ///
        /// Scene A1_PainterExit2
        #[default = false]
        scene_a1_painter_exit2 = "A1_PainterExit2",

        /// The Big Walk
        ///
        /// Scene A1_GameDesigner1
        #[default = false]
        scene_a1_game_designer1 = "A1_GameDesigner1",

        /// Main Avenue
        ///
        /// Scene A1_AvenueEast
        #[default = false]
        scene_a1_avenue_east = "A1_AvenueEast",

        /// Central Market
        ///
        /// Scene A1_Market
        #[default = false]
        scene_a1_market = "A1_Market",

        /// Piano Alley
        ///
        /// Scene A1_MusicianPath4
        #[default = false]
        scene_a1_musician_path4 = "A1_MusicianPath4",

        /// Corner's Club
        ///
        /// Scene A1_AvenueEastTurn
        #[default = false]
        scene_a1_avenue_east_turn = "A1_AvenueEastTurn",

        /// Beautiful Horizon Avenue
        ///
        /// Scene A1_MinibossAvenue
        #[default = false]
        scene_a1_miniboss_avenue = "A1_MinibossAvenue",

        /// Thommaz's House
        ///
        /// Scene A1_MusicianHouse
        #[default = false]
        scene_a1_musician_house = "A1_MusicianHouse",

        /// Theater Arena
        ///
        /// Scene A1_GD2
        #[default = false]
        scene_a1_gd2 = "A1_GD2",

        /// One-way Street
        ///
        /// Scene A1_GD3
        #[default = false]
        scene_a1_gd3 = "A1_GD3",

        /// Writer's Flow
        ///
        /// Scene A1_GD10
        #[default = false]
        scene_a1_gd10 = "A1_GD10",

        /// Dancing Triplets
        ///
        /// Scene A1_GD11
        #[default = false]
        scene_a1_gd11 = "A1_GD11",

        /// Mind Cross
        ///
        /// Scene A1_GD12
        #[default = false]
        scene_a1_gd12 = "A1_GD12",

        /// The Dalvian Star
        ///
        /// Scene A1_GD8
        #[default = false]
        scene_a1_gd8 = "A1_GD8",

        /// Buritis
        ///
        /// Scene A1_GD4
        #[default = false]
        scene_a1_gd4 = "A1_GD4",

        /// Corral Mountain Range
        ///
        /// Scene A1_GD13
        #[default = false]
        scene_a1_gd13 = "A1_GD13",

        /// Jonny B. Cave
        ///
        /// Scene A1_GD14
        #[default = false]
        scene_a1_gd14 = "A1_GD14",

        /// Concrete Cave
        ///
        /// Scene A1_GD15
        #[default = false]
        scene_a1_gd15 = "A1_GD15",

        /// Temple of Creation (Pre Boss)
        ///
        /// Scene A1_PreBoss
        #[default = false]
        scene_a1_pre_boss = "A1_PreBoss",

        /// Temple of Creation (Boss Fight)
        ///
        /// Scene A1_BossFightRoom
        #[default = false]
        scene_a1_boss_fight_room = "A1_BossFightRoom",

        /// Temple of Creation (Stone Room)
        ///
        /// Scene A1_CreationStoneRoom
        #[default = false]
        scene_a1_creation_stone_room = "A1_CreationStoneRoom",

        /// Main Gate
        ///
        /// Scene A1_CityMainEntrance
        #[default = false]
        scene_a1_city_main_entrance = "A1_CityMainEntrance",

        /// Village Outskirts
        ///
        /// Scene A1_BridgePath
        #[default = false]
        scene_a1_bridge_path = "A1_BridgePath",

        /// Confusion Ruins
        ///
        /// Scene AB_Backtrack4
        #[default = false]
        scene_ab_backtrack4 = "AB_Backtrack4",

        /// The Colossal Bridge
        ///
        /// Scene AB_Bridge
        #[default = false]
        scene_ab_bridge = "AB_Bridge",

        /// The Traveler's Corridor
        ///
        /// Scene AB_ToFieldCamp1
        #[default = false]
        scene_ab_to_field_camp1 = "AB_ToFieldCamp1",

        /// Capital Outskirts
        ///
        /// Scene AB_FieldCamp
        #[default = false]
        scene_ab_field_camp = "AB_FieldCamp",

        /// Forest of Encoders
        ///
        /// Scene AB_ToDesert1
        #[default = false]
        scene_ab_to_desert1 = "AB_ToDesert1",

        /// Hoarders Woods
        ///
        /// Scene AB_ToDesert2
        #[default = false]
        scene_ab_to_desert2 = "AB_ToDesert2",

        /// Reclamation Grove
        ///
        /// Scene AB_ToDesert3
        #[default = false]
        scene_ab_to_desert3 = "AB_ToDesert3",

        /// Overcast Gate Ruins
        ///
        /// Scene A2_Boss
        #[default = false]
        scene_a2_boss = "A2_Boss",

        /// Corridor of Conscious Recollections
        ///
        /// Scene A2_DesertJumper
        #[default = false]
        scene_a2_desert_jumper = "A2_DesertJumper",

        /// View of Solitude
        ///
        /// Scene A2_DesertCamp
        #[default = false]
        scene_a2_desert_camp = "A2_DesertCamp",

        /// The Wanderer Archives
        ///
        /// Scene A2_DesertCrawler1
        #[default = false]
        scene_a2_desert_crawler1 = "A2_DesertCrawler1",

        /// The Drifter Archives
        ///
        /// Scene A2_Lotus
        #[default = false]
        scene_a2_lotus = "A2_Lotus",

        /// Eldarian Army Outpost
        ///
        /// Scene A2_RBPlatformTutorial
        #[default = false]
        scene_a2_rb_platform_tutorial = "A2_RBPlatformTutorial",

        /// Main Warehouse
        ///
        /// Scene A2_LotusCorridor
        #[default = false]
        scene_a2_lotus_corridor = "A2_LotusCorridor",

        /// Archive of the Regretful
        ///
        /// Scene A2_AnteChamberUpRight
        #[default = false]
        scene_a2_ante_chamber_up_right = "A2_AnteChamberUpRight",

        /// Museum Main Hall
        ///
        /// Scene A2_ToInsideShortcut
        #[default = false]
        scene_a2_to_inside_shortcut = "A2_ToInsideShortcut",

        /// Auditorium
        ///
        /// Scene A2_ToInside3
        #[default = false]
        scene_a2_to_inside3 = "A2_ToInside3",

        /// Overburden Deposits
        ///
        /// Scene A2_ToInside4
        #[default = false]
        scene_a2_to_inside4 = "A2_ToInside4",

        /// Museum Side Entrance
        ///
        /// Scene A2_ToInside5
        #[default = false]
        scene_a2_to_inside5 = "A2_ToInside5",

        /// C for Curiosity and Creation
        ///
        /// Scene A2_Threeway
        #[default = false]
        scene_a2_threeway = "A2_Threeway",

        /// Kid's Lecture House
        ///
        /// Scene A2_DesertAdvancedCamp
        #[default = false]
        scene_a2_desert_advanced_camp = "A2_DesertAdvancedCamp",

        /// Tower Stairway
        ///
        /// Scene A2_BeforeWriter
        #[default = false]
        scene_a2_before_writer = "A2_BeforeWriter",

        /// The Tower
        ///
        /// Scene A2_WriterRoom
        #[default = false]
        scene_a2_writer_room = "A2_WriterRoom",

        /// Writer's Lone Balcony
        ///
        /// Scene A2_BarrierMiniboss
        #[default = false]
        scene_a2_barrier_miniboss = "A2_BarrierMiniboss",

        /// Remembrance Cliff
        ///
        /// Scene A2_RemembranceStoneAltar
        #[default = false]
        scene_a2_remembrance_stone_altar = "A2_RemembranceStoneAltar",

        /// F for Findings and Fun
        ///
        /// Scene A2_ToOutside1
        #[default = false]
        scene_a2_to_outside1 = "A2_ToOutside1",

        /// Quarter of a Distant Love
        ///
        /// Scene A2_RemembranceWeaponRoom
        #[default = false]
        scene_a2_remembrance_weapon_room = "A2_RemembranceWeaponRoom",

        /// B for Bruises and Betweens
        ///
        /// Scene A2_DesertCrawler3
        #[default = false]
        scene_a2_desert_crawler3 = "A2_DesertCrawler3",

        /// Corner of the Unasked Question
        ///
        /// Scene A2_ToOutside2
        #[default = false]
        scene_a2_to_outside2 = "A2_ToOutside2",

        /// S for Smiles and Scars
        ///
        /// Scene A2_ToInside1
        #[default = false]
        scene_a2_to_inside1 = "A2_ToInside1",

        /// Remember Good Moments
        ///
        /// Scene A2_ToRemembranceWeapon
        #[default = false]
        scene_a2_to_remembrance_weapon = "A2_ToRemembranceWeapon",

        /// Corruption Transform
        ///
        /// Scene AB_ToAF
        #[default = false]
        scene_ab_to_af = "AB_ToAF",

        /// The Golden Threat
        ///
        /// Scene AF_4
        #[default = false]
        scene_af_4 = "AF_4",

        /// The Golden Pride
        ///
        /// Scene AF_5
        #[default = false]
        scene_af_5 = "AF_5",

        /// Eldarian Gates
        ///
        /// Scene AF_FinalCamp
        #[default = false]
        scene_af_final_camp = "AF_FinalCamp",

        /// The Golden Honor
        ///
        /// Scene AF_6
        #[default = false]
        scene_af_6 = "AF_6",

        /// The Golden Menace
        ///
        /// Scene AF_7
        #[default = false]
        scene_af_7 = "AF_7",

        /// Reasoning Lock
        ///
        /// Scene A3_Camp
        #[default = false]
        scene_a3_camp = "A3_Camp",

        /// Logical Path
        ///
        /// Scene A3_ToIntention1
        #[default = false]
        scene_a3_to_intention1 = "A3_ToIntention1",

        /// The Golden Walkway
        ///
        /// Scene AF_8
        #[default = false]
        scene_af_8 = "AF_8",

        /// The Golden Storage
        ///
        /// Scene AF_9
        #[default = false]
        scene_af_9 = "AF_9",

        /// Intention Square
        ///
        /// Scene A3_ToIntention2
        #[default = false]
        scene_a3_to_intention2 = "A3_ToIntention2",

        /// Crib of Intention
        ///
        /// Scene A3_CribOfIntention
        #[default = false]
        scene_a3_crib_of_intention = "A3_CribOfIntention",

        /// Limits of Sanity
        ///
        /// Scene AB_Backtrack1
        #[default = false]
        scene_ab_backtrack1 = "AB_Backtrack1",

        /// The Golden Path
        ///
        /// Scene AF_3
        #[default = false]
        scene_af_3 = "AF_3",

        /// The Golden Weaponry
        ///
        /// Scene AF_2
        #[default = false]
        scene_af_2 = "AF_2",

        /// The Breach in the Wall
        ///
        /// Scene AF_1
        #[default = false]
        scene_af_1 = "AF_1",

        /// Capital Entrance
        ///
        /// Scene A3_CityEntrance
        #[default = false]
        scene_a3_city_entrance = "A3_CityEntrance",

        /// Side Turn
        ///
        /// Scene A3_Back7
        #[default = false]
        scene_a3_back7 = "A3_Back7",

        /// View of the Deep
        ///
        /// Scene A4_Entrance
        #[default = false]
        scene_a4_entrance = "A4_Entrance",

        /// Dream Lands (Circle)
        ///
        /// Scene A4_Circle
        #[default = false]
        scene_a4_circle = "A4_Circle",

        /// Dream Lands (Camp)
        ///
        /// Scene A4_DreamStone
        #[default = false]
        scene_a4_dream_stone = "A4_DreamStone",

        /// The Golden Angle
        ///
        /// Scene AF_12
        #[default = false]
        scene_af_12 = "AF_12",

        /// Path to Great Fortune
        ///
        /// Scene AF_MainRoom
        #[default = false]
        scene_af_main_room = "AF_MainRoom",

        /// ID Control
        ///
        /// Scene AF_D1
        #[default = false]
        scene_af_d1 = "AF_D1",

        /// Guardhouse
        ///
        /// Scene AF_D4
        #[default = false]
        scene_af_d4 = "AF_D4",

        /// Control Chamber
        ///
        /// Scene AF_Boss
        #[default = false]
        scene_af_boss = "AF_Boss",
    }
}
