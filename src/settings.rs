use asr::{
    settings::{gui::Title, Gui},
};

macro_rules! define_settings {
    (
        events: {
            $(
                $(#[doc = $event_doc:expr])?
                #[default = $event_default:expr]
                $event_field:ident = $event_id:expr,
            )*
        }
        scenes: {
            $(
                $(#[doc = $scene_doc:expr])?
                #[default = $scene_default:expr]
                $scene_field:ident = $scene_id:expr,
            )*
        }
    ) => {
        #[derive(Gui)]
        pub struct Settings {
            /// Generic Settings
            _generic_settings: Title,

            /// Remove Loading
            #[default = false]
            remove_loading: bool,
            
            /// Split on current scene name
            #[default = true]
            split_current_scene: bool,
            
            /// Split on current camp name
            #[default = false]
            split_saved_camp: bool,
            
            /// Try to split on end credits
            #[default = false]
            split_end_credits: bool,
            
            /// Events
            _event_settings: Title,
            // Generated event fields
            $(
                $(#[doc = $event_doc])?
                #[default = $event_default]
                $event_field: bool,
            )*
            
            /// Scenes
            _scene_settings: Title,
            // Generated scene fields
            $(
                $(#[doc = $scene_doc])?
                #[default = $scene_default]
                $scene_field: bool,
            )*
        }
        
        impl Settings {
            pub fn get_event_enabled(&self, event_id: i32) -> bool {
                match event_id {
                    $(
                        $event_id => self.$event_field,
                    )*
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
            
            pub fn get_event_name(event_id: i32) -> Option<&'static str> {
                match event_id {
                    $(
                        $event_id => Some(stringify!($event_field)),
                    )*
                    _ => None,
                }
            }
            
            // pub fn get_scene_name(scene_id: &str) -> Option<&'static str> {
            //     match scene_id {
            //         $(
            //             $scene_id => Some(stringify!($scene_field)),
            //         )*
            //         _ => None,
            //     }
            // }
        }
    };
}

// Define everything ONCE here - single invocation!
define_settings! {
    events: {
        /// Game started
        #[default = false]
        event_started = 0x1,
        
        /// Started credits
        #[default = true]
        event_ended = 0x2,
        
        /// Hit lever in Guardhouse
        #[default = false]
        event_opened_eldar_door = 0x3,
        
        /// Got first arrow of freedom
        #[default = true]
        event_opened_first_weapon_chest = 0x4,
        
        /// DLCF_EntranceAccessUnlocked
        #[default = false]
        event_dlcf_entrance_access_unlocked = 0x5,
        
        /// DLCF_FacedTheMasters
        #[default = false]
        event_dlcf_faced_the_masters = 0x6,
        
        /// DLCF_FreedNara
        #[default = false]
        event_dlcf_freed_nara = 0x7,
        
        /// DLCF_TimeFlagTrap
        #[default = false]
        event_dlcf_time_flag_trap = 0x8,
        
        /// DLCF_LazuliDeath
        #[default = false]
        event_dlcf_lazuli_death = 0x9,
        
        /// DLCF_FearEnded
        #[default = false]
        event_dlcf_fear_ended = 0xA,
        
        /// Final Boss Kill
        #[default = false]
        event_final_boss_kill = 0x31,
        
        /// Killed Augustus
        #[default = true]
        event_boss_kill_1 = 0x32,
        
        /// Killed Belia
        #[default = false]
        event_boss_kill_2 = 0x33,
        
        /// Boss Kill 3
        #[default = false]
        event_boss_kill_3 = 0x34,
        
        /// Boss Kill 4
        #[default = false]
        event_boss_kill_4 = 0x35,
        
        /// Boss Kill 5
        #[default = false]
        event_boss_kill_5 = 0x36,
        
        /// Killed Eldar tv form
        #[default = true]
        event_boss_kill_6 = 0x37,
        
        /// Boss Kill 7
        #[default = false]
        event_boss_kill_7 = 0x38,
        
        /// Killed tank
        #[default = true]
        event_miniboss_kill_1 = 0x39,
        
        /// Killed rock of remembrance guardian
        #[default = false]
        event_miniboss_kill_2 = 0x3A,
        
        /// Killed stone of intention guardian
        #[default = false]
        event_miniboss_kill_3 = 0x3B,
        
        /// Miniboss Kill 4
        #[default = false]
        event_miniboss_kill_4 = 0x3C,
        
        /// Got stone of creation
        #[default = false]
        event_pu_stone_creation = 0x64,
        
        /// Got rock of Remembrance
        #[default = true]
        event_pu_stone_remembrance = 0x65,
        
        /// Got stone of Intention
        #[default = true]
        event_pu_stone_intention = 0x66,
        
        /// Got pearl of dreams
        #[default = true]
        event_pu_stone_dreams = 0x67,
        
        /// PU_DLCF_FinalKey
        #[default = false]
        event_pu_dlcf_final_key = 0x6C,
        
        /// PU_Health
        #[default = false]
        event_pu_health = 0x6E,
        
        /// Got first weapon
        #[default = false]
        event_pu_ammo = 0x6F,
        
        /// Got first health flask
        #[default = true]
        event_pu_health_flask = 0x70,
        
        /// Got map
        #[default = false]
        event_pu_map = 0x71,
        
        /// PU_Shield
        #[default = false]
        event_pu_shield = 0x72,
        
        /// Got first mana flask
        #[default = false]
        event_pu_mana_flask = 0x73,
        
        /// Got first health flask upgrade
        #[default = false]
        event_pu_health_flask_upgrade = 0x74,
        
        /// Got first mana flask upgrade
        #[default = false]
        event_pu_mana_flask_upgrade = 0x75,
        
        /// PU_Money
        #[default = false]
        event_pu_money = 0x76,
        
        /// PU_DandaraArrow
        #[default = false]
        event_pu_dandara_arrow = 0x77,
        
        /// Got first mana weapon
        #[default = false]
        event_pu_mana_weapon = 0x78,
        
        /// PU_Money_Fear
        #[default = false]
        event_pu_money_fear = 0x79,
        
        /// PU_SuperDandara
        #[default = false]
        event_pu_super_dandara = 0x7A,
        
        /// PU_FearWeapon
        #[default = false]
        event_pu_fear_weapon = 0x7B,
        
        /// Got missile weapon
        #[default = true]
        event_weapon_missile = 0x96,
        
        /// Weapon_EnergyBall
        #[default = false]
        event_weapon_energy_ball = 0x97,
        
        /// Got memory shaft
        #[default = true]
        event_weapon_remembrance = 0x98,
        
        /// Weapon_Bounce
        #[default = false]
        event_weapon_bounce = 0x99,
        
        /// Weapon_Boomerang
        #[default = false]
        event_weapon_boomerang = 0x9A,
        
        /// Weapon_Vaccum
        #[default = false]
        event_weapon_vaccum = 0x9B,
        
        /// Weapon_WaterBomb
        #[default = false]
        event_weapon_water_bomb = 0x9C,
        
        /// Weapon_Teleport
        #[default = false]
        event_weapon_teleport = 0x9D,
        
        /// Weapon_Firewall
        #[default = false]
        event_weapon_firewall = 0x9E,
        
        /// Activated painter
        #[default = false]
        event_d_painter = 0xC8,
        
        /// Activated musician
        #[default = false]
        event_d_musician = 0xC9,
        
        /// Talked to Lazuli at bridge
        #[default = false]
        event_d_creat_guard_bridge = 0xCA,
        
        /// Talked to Eldar 1
        #[default = false]
        event_d_eldar1 = 0xCB,
        
        /// D_DLCF_MastersIntroduction
        #[default = false]
        event_d_dlcf_masters_introduction = 0xCC,
        
        /// D_DLCF_ExplorerFinish
        #[default = false]
        event_d_dlcf_explorer_finish = 0xCD,
        
        /// D_DLCF_NobleFinish
        #[default = false]
        event_d_dlcf_noble_finish = 0xCE,
        
        /// D_DLCF_PersistentFinish
        #[default = false]
        event_d_dlcf_persistent_finish = 0xCF,
        
        /// D_DLCF_ExplorerStart
        #[default = false]
        event_d_dlcf_explorer_start = 0xD0,
        
        /// D_DLCF_NobleStart
        #[default = false]
        event_d_dlcf_noble_start = 0xD1,
        
        /// D_DLCF_PersistentStart
        #[default = false]
        event_d_dlcf_persistent_start = 0xD2,
        
        /// HUD_Unlock
        #[default = false]
        event_hud_unlock = 0xFA,
        
        /// HUD_TutorialClear
        #[default = false]
        event_hud_tutorial_clear = 0xFB,
        
        /// Talked to painter
        #[default = false]
        event_char_painter = 0x12C,
        
        /// Talked to musician
        #[default = false]
        event_char_musician = 0x12D,
        
        /// Talked to Jhonny
        #[default = false]
        event_char_game_dev = 0x12E,
        
        /// Discovered first camp
        #[default = false]
        event_char_boss1_no_name = 0x12F,
        
        /// Started Augustus fight
        #[default = false]
        event_char_a1_boss = 0x130,
        
        /// Char_A2Boss
        #[default = false]
        event_char_a2_boss = 0x131,
        
        /// Char_Writer
        #[default = false]
        event_char_writer = 0x132,
        
        /// Talked to Eldar 1
        #[default = false]
        event_char_af_boss1 = 0x133,
        
        /// Started Eldar 2
        #[default = false]
        event_char_af_boss2 = 0x134,
        
        /// Char_HeartCourage
        #[default = false]
        event_char_heart_courage = 0x135,
        
        /// Char_DLCF_Boss
        #[default = false]
        event_char_dlcf_boss = 0x136,
        
        /// Char_DLCF_Explorer
        #[default = false]
        event_char_dlcf_explorer = 0x137,
        
        /// Char_DLCF_Noble
        #[default = false]
        event_char_dlcf_noble = 0x138,
        
        /// Char_DLCF_Persistent
        #[default = false]
        event_char_dlcf_persistent = 0x139,
        
        /// Talked to Lazuli at Stone of Creation
        #[default = false]
        event_char_lazuli = 0x13A,
        
        /// Char_Lazuli_NoName
        #[default = false]
        event_char_lazuli_no_name = 0x13B,
        
        /// Char_DLCF_Nara
        #[default = false]
        event_char_dlcf_nara = 0x13C,
        
        /// Char_DLCF_ElderDandara
        #[default = false]
        event_char_dlcf_elder_dandara = 0x13D,
        
        /// Augustus door shake cutscene
        #[default = false]
        event_cutscene_favela_hub = 0x1F4,
        
        /// Cutscene_CampDiscovery
        #[default = false]
        event_cutscene_camp_discovery = 0x1F5,
    }
    scenes: {
        /// Crib of Creation
        #[default = false]
        scene_a1_great_ruins = "A1_GreatRuins",
        
        /// Creation Meadow
        #[default = false]
        scene_a1_forest_edge = "A1_ForestEdge",
        
        /// Side Gate
        #[default = false]
        scene_a1_city_entrance = "A1_CityEntrance",
        
        /// The Village Center
        #[default = false]
        scene_a1_favela_hub = "A1_FavelaHub",
        
        /// Paint Well
        #[default = false]
        scene_a1_painter_path1 = "A1_PainterPath1",
        
        /// Abandoned House
        #[default = false]
        scene_a1_painter_path2 = "A1_PainterPath2",
        
        /// Community Street
        #[default = false]
        scene_a1_painter_path3 = "A1_PainterPath3",
        
        /// The Rich and the Poor
        #[default = false]
        scene_a1_painter_path4 = "A1_PainterPath4",
        
        /// Tarsila's House
        #[default = false]
        scene_a1_painter_house = "A1_PainterHouse",
        
        /// Backyard
        #[default = false]
        scene_a1_painter_exit1 = "A1_PainterExit1",
        
        /// Sewers
        #[default = false]
        scene_a1_painter_exit2 = "A1_PainterExit2",
        
        /// The Big Walk
        #[default = false]
        scene_a1_game_designer1 = "A1_GameDesigner1",
        
        /// Main Avenue
        #[default = false]
        scene_a1_avenue_east = "A1_AvenueEast",
        
        /// Central Market
        #[default = false]
        scene_a1_market = "A1_Market",
        
        /// Piano Alley
        #[default = false]
        scene_a1_musician_path4 = "A1_MusicianPath4",
        
        /// Corner's Club
        #[default = false]
        scene_a1_avenue_east_turn = "A1_AvenueEastTurn",
        
        /// Beautiful Horizon Avenue
        #[default = false]
        scene_a1_miniboss_avenue = "A1_MinibossAvenue",
        
        /// Thommaz's House
        #[default = false]
        scene_a1_musician_house = "A1_MusicianHouse",
        
        /// Theater Arena
        #[default = false]
        scene_a1_gd2 = "A1_GD2",
        
        /// One-way Street
        #[default = false]
        scene_a1_gd3 = "A1_GD3",
        
        /// Writer's Flow
        #[default = false]
        scene_a1_gd10 = "A1_GD10",
        
        /// Dancing Triplets
        #[default = false]
        scene_a1_gd11 = "A1_GD11",
        
        /// Mind Cross
        #[default = false]
        scene_a1_gd12 = "A1_GD12",
        
        /// The Dalvian Star
        #[default = false]
        scene_a1_gd8 = "A1_GD8",
        
        /// Buritis
        #[default = false]
        scene_a1_gd4 = "A1_GD4",
        
        /// Corral Mountain Range
        #[default = false]
        scene_a1_gd13 = "A1_GD13",
        
        /// Jonny B. Cave
        #[default = false]
        scene_a1_gd14 = "A1_GD14",
        
        /// Concrete Cave
        #[default = false]
        scene_a1_gd15 = "A1_GD15",
        
        /// Temple of Creation (Pre Boss)
        #[default = false]
        scene_a1_pre_boss = "A1_PreBoss",
        
        /// Temple of Creation (Boss Fight)
        #[default = false]
        scene_a1_boss_fight_room = "A1_BossFightRoom",
        
        /// Temple of Creation (Stone Room)
        #[default = false]
        scene_a1_creation_stone_room = "A1_CreationStoneRoom",
        
        /// Main Gate
        #[default = false]
        scene_a1_city_main_entrance = "A1_CityMainEntrance",
        
        /// Village Outskirts
        #[default = false]
        scene_a1_bridge_path = "A1_BridgePath",
        
        /// Confusion Ruins
        #[default = false]
        scene_ab_backtrack4 = "AB_Backtrack4",
        
        /// The Colossal Bridge
        #[default = false]
        scene_ab_bridge = "AB_Bridge",
        
        /// The Traveler's Corridor
        #[default = false]
        scene_ab_to_field_camp1 = "AB_ToFieldCamp1",
        
        /// Capital Outskirts
        #[default = false]
        scene_ab_field_camp = "AB_FieldCamp",
        
        /// Forest of Encoders
        #[default = false]
        scene_ab_to_desert1 = "AB_ToDesert1",
        
        /// Hoarders Woods
        #[default = false]
        scene_ab_to_desert2 = "AB_ToDesert2",
        
        /// Reclamation Grove
        #[default = false]
        scene_ab_to_desert3 = "AB_ToDesert3",
        
        /// Overcast Gate Ruins
        #[default = false]
        scene_a2_boss = "A2_Boss",
        
        /// Corridor of Conscious Recollections
        #[default = false]
        scene_a2_desert_jumper = "A2_DesertJumper",
        
        /// View of Solitude
        #[default = false]
        scene_a2_desert_camp = "A2_DesertCamp",
        
        /// The Wanderer Archives
        #[default = false]
        scene_a2_desert_crawler1 = "A2_DesertCrawler1",
        
        /// The Drifter Archives
        #[default = false]
        scene_a2_lotus = "A2_Lotus",
        
        /// Eldarian Army Outpost
        #[default = false]
        scene_a2_rb_platform_tutorial = "A2_RBPlatformTutorial",
        
        /// Main Warehouse
        #[default = false]
        scene_a2_lotus_corridor = "A2_LotusCorridor",
        
        /// Archive of the Regretful
        #[default = false]
        scene_a2_ante_chamber_up_right = "A2_AnteChamberUpRight",
        
        /// Museum Main Hall
        #[default = false]
        scene_a2_to_inside_shortcut = "A2_ToInsideShortcut",
        
        /// Auditorium
        #[default = false]
        scene_a2_to_inside3 = "A2_ToInside3",
        
        /// Overburden Deposits
        #[default = false]
        scene_a2_to_inside4 = "A2_ToInside4",
        
        /// Museum Side Entrance
        #[default = false]
        scene_a2_to_inside5 = "A2_ToInside5",
        
        /// C for Curiosity and Creation
        #[default = false]
        scene_a2_threeway = "A2_Threeway",
        
        /// Kid's Lecture House
        #[default = false]
        scene_a2_desert_advanced_camp = "A2_DesertAdvancedCamp",
        
        /// Tower Stairway
        #[default = false]
        scene_a2_before_writer = "A2_BeforeWriter",
        
        /// The Tower
        #[default = false]
        scene_a2_writer_room = "A2_WriterRoom",
        
        /// Writer's Lone Balcony
        #[default = false]
        scene_a2_barrier_miniboss = "A2_BarrierMiniboss",
        
        /// Remembrance Cliff
        #[default = false]
        scene_a2_remembrance_stone_altar = "A2_RemembranceStoneAltar",
        
        /// F for Findings and Fun
        #[default = false]
        scene_a2_to_outside1 = "A2_ToOutside1",
        
        /// Quarter of a Distant Love
        #[default = false]
        scene_a2_remembrance_weapon_room = "A2_RemembranceWeaponRoom",
        
        /// B for Bruises and Betweens
        #[default = false]
        scene_a2_desert_crawler3 = "A2_DesertCrawler3",
        
        /// Corner of the Unasked Question
        #[default = false]
        scene_a2_to_outside2 = "A2_ToOutside2",
        
        /// S for Smiles and Scars
        #[default = false]
        scene_a2_to_inside1 = "A2_ToInside1",
        
        /// Remember Good Moments
        #[default = false]
        scene_a2_to_remembrance_weapon = "A2_ToRemembranceWeapon",
        
        /// Corruption Transform
        #[default = false]
        scene_ab_to_af = "AB_ToAF",
        
        /// The Golden Threat
        #[default = false]
        scene_af_4 = "AF_4",
        
        /// The Golden Pride
        #[default = false]
        scene_af_5 = "AF_5",
        
        /// Eldarian Gates
        #[default = false]
        scene_af_final_camp = "AF_FinalCamp",
        
        /// The Golden Honor
        #[default = false]
        scene_af_6 = "AF_6",
        
        /// The Golden Menace
        #[default = false]
        scene_af_7 = "AF_7",
        
        /// Reasoning Lock
        #[default = false]
        scene_a3_camp = "A3_Camp",
        
        /// Logical Path
        #[default = false]
        scene_a3_to_intention1 = "A3_ToIntention1",
        
        /// The Golden Walkway
        #[default = false]
        scene_af_8 = "AF_8",
        
        /// The Golden Storage
        #[default = false]
        scene_af_9 = "AF_9",
        
        /// Intention Square
        #[default = false]
        scene_a3_to_intention2 = "A3_ToIntention2",
        
        /// Crib of Intention
        #[default = false]
        scene_a3_crib_of_intention = "A3_CribOfIntention",
        
        /// Limits of Sanity
        #[default = false]
        scene_ab_backtrack1 = "AB_Backtrack1",
        
        /// The Golden Path
        #[default = false]
        scene_af_3 = "AF_3",
        
        /// The Golden Weaponry
        #[default = false]
        scene_af_2 = "AF_2",
        
        /// The Breach in the Wall
        #[default = false]
        scene_af_1 = "AF_1",
        
        /// Capital Entrance
        #[default = false]
        scene_a3_city_entrance = "A3_CityEntrance",
        
        /// Side Turn
        #[default = false]
        scene_a3_back7 = "A3_Back7",
        
        /// View of the Deep
        #[default = false]
        scene_a4_entrance = "A4_Entrance",
        
        /// Dream Lands (Circle)
        #[default = false]
        scene_a4_circle = "A4_Circle",
        
        /// Dream Lands (Camp)
        #[default = false]
        scene_a4_dream_stone = "A4_DreamStone",
        
        /// The Golden Angle
        #[default = false]
        scene_af_12 = "AF_12",
        
        /// Path to Great Fortune
        #[default = false]
        scene_af_main_room = "AF_MainRoom",
        
        /// ID Control
        #[default = false]
        scene_af_d1 = "AF_D1",
        
        /// Guardhouse
        #[default = false]
        scene_af_d4 = "AF_D4",
        
        /// Control Chamber
        #[default = false]
        scene_af_boss = "AF_Boss",
    }
}
