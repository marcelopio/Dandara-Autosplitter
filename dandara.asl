state("Dandara", "steam")
{
    float playedTime : "UnityPlayer.dll", 0x010A1120, 0x10, 0x7E0, 0x54, 0x7B4, 0x0, 0x4c;
    //string255 currentRoomNameID : "UnityPlayer.dll", 0x010A1120, 0x10, 0x7E0, 0x54, 0x7B4, 0x0, 0x38, 0xc;
    string255 currentScene : "UnityPlayer.dll", 0x010A1120, 0x10, 0x7E0, 0x54, 0x7B4, 0x0, 0x3c, 0xc;
    string255 baseCampScene : "UnityPlayer.dll", 0x010A1120, 0x10, 0x7E0, 0x54, 0x7B4, 0x0, 0x48, 0xc;
    byte isTransitioning : "UnityPlayer.dll", 0x010A1120, 0x10, 0x7E0, 0x54, 0x7B4, 0x0, 0x65;
    
    int storyManager : "UnityPlayer.dll", 0x0107DD14, 0xa8, 0x4, 0x8c, 0x54, 0x290, 0x10;
    int ocurredEvents : "UnityPlayer.dll", 0x0107DD14, 0xa8, 0x4, 0x8c, 0x54, 0x290, 0x10, 0x10;
    int arrayEvents : "UnityPlayer.dll", 0x0107DD14, 0xa8, 0x4, 0x8c, 0x54, 0x290, 0x10, 0x10, 0x10;
    int numEvents : "UnityPlayer.dll", 0x0107DD14, 0xa8, 0x4, 0x8c, 0x54, 0x290, 0x10, 0x10, 0x24;
}

state("Dandara", "epic")
{
    float playedTime : "mono.dll", 0x001F72C4, 0x54, 0x280, 0x0, 0x8, 0xc8, 0x3c, 0x4c;
    //string255 currentRoomNameID : "mono.dll", 0x001F72C4, 0x54, 0x280, 0x0, 0x8, 0xc8, 0x3c, 0x38, 0xc;
    string255 currentScene : "mono.dll", 0x001F72C4, 0x54, 0x280, 0x0, 0x8, 0xc8, 0x3c, 0x3c, 0xc;
    string255 baseCampScene : "mono.dll", 0x001F72C4, 0x54, 0x280, 0x0, 0x8, 0xc8, 0x3c, 0x48, 0xc;
    byte isTransitioning : "mono.dll", 0x001F72C4, 0x54, 0x280, 0x0, 0x8, 0xc8, 0x3c, 0x65;
    
    int storyManager : "UnityPlayer.dll", 0x0107DD14, 0xa8, 0x4, 0x8c, 0x54, 0x290, 0x10;
    int ocurredEvents : "UnityPlayer.dll", 0x0107DD14, 0xa8, 0x4, 0x8c, 0x54, 0x290, 0x10, 0x10;
    int arrayEvents : "UnityPlayer.dll", 0x0107DD14, 0xa8, 0x4, 0x8c, 0x54, 0x290, 0x10, 0x10, 0x10;
    int numEvents : "UnityPlayer.dll", 0x0107DD14, 0xa8, 0x4, 0x8c, 0x54, 0x290, 0x10, 0x10, 0x24;
}


startup {
    vars.aslName = "Dandara";
    if(timer.CurrentTimingMethod == TimingMethod.RealTime){
        
        var timingMessage = MessageBox.Show(
            "This game uses Game Time as the main timing method.\n"+
            "LiveSplit is currently set to show Real Time (time INCLUDING loads).\n"+
            "Would you like the timing method to be set to Game Time for you?",
            vars.aslName+" | LiveSplit",
            MessageBoxButtons.YesNo,MessageBoxIcon.Question
        );
        if (timingMessage == DialogResult.Yes)
            timer.CurrentTimingMethod = TimingMethod.GameTime;
    }
    
    vars.ResetVars = (Action)(() => {
        vars.currentStoryEvents = new HashSet<int>();
        vars.currentScenes = new HashSet<string>();
    });
    
    vars.ResetVars();

    vars.timerResetVars = (EventHandler)((s, e) => {
        vars.ResetVars();
    });
    timer.OnStart += vars.timerResetVars;
    
    vars.storyEventDictionary = new Dictionary<int, string>();
    Action<int, string, bool, string> AddEventSplit = (key, name, active, description) => {
        if(description == string.Empty)
          description = "Unknown event";
        settings.Add(name, active, description + " (" + name + ")", "event");
        settings.SetToolTip(name, description);
        vars.storyEventDictionary[key] = name;
    };

    vars.sceneDictionary = new Dictionary<int, string>();
    Action<int, string, bool, string> AddSceneSplit = (key, name, active, description) => {
        settings.Add(name, active, description + " (" + name + ")", "scene");
        vars.sceneDictionary[key] = name;
    };

    settings.Add("event", true, "Events");
    settings.Add("scene", true, "Scenes");
    settings.Add("remove_loading", false, "Remove Loading");
    settings.Add("split_current_scene", true, "Split on current scene name");
    settings.Add("split_saved_camp", false, "Split on current camp name, does not work with current scene enabled");
    
    //Thanks johno for theses descriptions
    
    AddEventSplit(0x1, "Started", false, "Game started");
    AddEventSplit(0x2, "Ended", true, "Started credits");
    AddEventSplit(0x3, "OpenedEldarDoor", false, "Hit lever in Guardhouse");
    AddEventSplit(0x4, "OpenedFirstWeaponChest", true, "Got first arrow of freedom");
    AddEventSplit(0x5, "DLCF_EntranceAccessUnlocked", false, "");
    AddEventSplit(0x6, "DLCF_FacedTheMasters", false, "");
    AddEventSplit(0x7, "DLCF_FreedNara", false, "");
    AddEventSplit(0x8, "DLCF_TimeFlagTrap", false, "");
    AddEventSplit(0x9, "DLCF_LazuliDeath", false, "");
    AddEventSplit(0xA, "DLCF_FearEnded", false, "");
    AddEventSplit(0x31, "FinalBoss_Kill", false, "");
    AddEventSplit(0x32, "Boss_Kill_1", true, "Killed Augustus");
    AddEventSplit(0x33, "Boss_Kill_2", false, "Killed Belia");
    AddEventSplit(0x34, "Boss_Kill_3", false, "");
    AddEventSplit(0x35, "Boss_Kill_4", false, "");
    AddEventSplit(0x36, "Boss_Kill_5", false, "");
    AddEventSplit(0x37, "Boss_Kill_6", true, "Killed Eldar tv form");
    AddEventSplit(0x38, "Boss_Kill_7", false, "");
    AddEventSplit(0x39, "Miniboss_Kill_1", true, "Killed tank");
    AddEventSplit(0x3A, "Miniboss_Kill_2", false, "Killed rock of remembrance guardian");
    AddEventSplit(0x3B, "Miniboss_Kill_3", false, "Killed stone of intention guardian");
    AddEventSplit(0x3C, "Miniboss_Kill_4", false, "");
    AddEventSplit(0x64, "PU_Stone_Creation", false, "Got stone of creation");
    AddEventSplit(0x65, "PU_Stone_Remembrance", true, "Got rock of Remembrance");
    AddEventSplit(0x66, "PU_Stone_Intention", true, "Got stone of Intention");
    AddEventSplit(0x67, "PU_Stone_Dreams", true, "Got pearl of dreams");
    AddEventSplit(0x6C, "PU_DLCF_FinalKey", false, "");
    AddEventSplit(0x6E, "PU_Health", false, "");
    AddEventSplit(0x6F, "PU_Ammo", false, "Got first weapon");
    AddEventSplit(0x70, "PU_HealthFlask", true, "Got first health flask");
    AddEventSplit(0x71, "PU_Map", false, "Got map");
    AddEventSplit(0x72, "PU_Shield", false, "");
    AddEventSplit(0x73, "PU_ManaFlask", false, "Got first mana flask");
    AddEventSplit(0x74, "PU_HealthFlaskUpgrade", false, "Got first health flask");
    AddEventSplit(0x75, "PU_ManaFlaskUpgrade", false, "Got first mana flask");
    AddEventSplit(0x76, "PU_Money", false, "");
    AddEventSplit(0x77, "PU_DandaraArrow", false, "");
    AddEventSplit(0x78, "PU_ManaWeapon", false, "Got first weapon");
    AddEventSplit(0x79, "PU_Money_Fear", false, "");
    AddEventSplit(0x7A, "PU_SuperDandara", false, "");
    AddEventSplit(0x7B, "PU_FearWeapon", false, "");
    AddEventSplit(0x96, "Weapon_Missile", true, "Got missile weapon");
    AddEventSplit(0x97, "Weapon_EnergyBall", false, "");
    AddEventSplit(0x98, "Weapon_Remembrance", true, "Got memory shaft");
    AddEventSplit(0x99, "Weapon_Bounce", false, "");
    AddEventSplit(0x9A, "Weapon_Boomerang", false, "");
    AddEventSplit(0x9B, "Weapon_Vaccum", false, "");
    AddEventSplit(0x9C, "Weapon_WaterBomb", false, "");
    AddEventSplit(0x9D, "Weapon_Teleport", false, "");
    AddEventSplit(0x9E, "Weapon_Firewall", false, "");
    AddEventSplit(0xC8, "D_Painter", false, "Activated painter");
    AddEventSplit(0xC9, "D_Musician", false, "Activated musician");
    AddEventSplit(0xCA, "D_CreatGuardBridge", false, "Talked to Lazuli at bridge");
    AddEventSplit(0xCB, "D_Eldar1", false, "Talked to Eldar 1");
    AddEventSplit(0xCC, "D_DLCF_MastersIntroduction", false, "");
    AddEventSplit(0xCD, "D_DLCF_ExplorerFinish", false, "");
    AddEventSplit(0xCE, "D_DLCF_NobleFinish", false, "");
    AddEventSplit(0xCF, "D_DLCF_PersistentFinish", false, "");
    AddEventSplit(0xD0, "D_DLCF_ExplorerStart", false, "");
    AddEventSplit(0xD1, "D_DLCF_NobleStart", false, "");
    AddEventSplit(0xD2, "D_DLCF_PersistentStart", false, "");
    AddEventSplit(0xFA, "HUD_Unlock", false, "");
    AddEventSplit(0xFB, "HUD_TutorialClear", false, "");
    AddEventSplit(0x12C, "Char_Painter", false, "Talked to painter");
    AddEventSplit(0x12D, "Char_Musician", false, "Talked to musician");
    AddEventSplit(0x12E, "Char_GameDev", false, "Talked to Jhonny");
    AddEventSplit(0x12F, "Char_Boss1_NoName", false, "Discovered first camp");
    AddEventSplit(0x130, "Char_A1Boss", false, "Started Augustus fight");
    AddEventSplit(0x131, "Char_A2Boss", false, "");
    AddEventSplit(0x132, "Char_Writer", false, "");
    AddEventSplit(0x133, "Char_AFBoss1", false, "Talked to Eldar 1");
    AddEventSplit(0x134, "Char_AFBoss2", false, "Started Eldar 2");
    AddEventSplit(0x135, "Char_HeartCourage", false, "");
    AddEventSplit(0x136, "Char_DLCF_Boss", false, "");
    AddEventSplit(0x137, "Char_DLCF_Explorer", false, "");
    AddEventSplit(0x138, "Char_DLCF_Noble", false, "");
    AddEventSplit(0x139, "Char_DLCF_Persistent", false, "");
    AddEventSplit(0x13A, "Char_Lazuli", false, "Talked to Lazuli at Stone of Creation");
    AddEventSplit(0x13B, "Char_Lazuli_NoName", false, "");
    AddEventSplit(0x13C, "Char_DLCF_Nara", false, "");
    AddEventSplit(0x13D, "Char_DLCF_ElderDandara", false, "");
    AddEventSplit(0x1F4, "Cutscene_FavelaHUB", false, "Augustus door shake cutscene");
    AddEventSplit(0x1F5, "Cutscene_CampDiscovery", false, "");

    AddSceneSplit(0x301, "A1_GreatRuins", false, "Crib of Creation");
    AddSceneSplit(0x302, "A1_ForestEdge", false, "Creation Meadow");
    AddSceneSplit(0x303, "A1_CityEntrance", false, "Side Gate");
    AddSceneSplit(0x304, "A1_FavelaHub", false, "The Village Center");
    AddSceneSplit(0x305, "A1_PainterPath1", false, "Paint Well");
    AddSceneSplit(0x306, "A1_PainterPath2", false, "Abandoned House");
    AddSceneSplit(0x307, "A1_PainterPath3", false, "Community Street");
    AddSceneSplit(0x308, "A1_PainterPath4", false, "The Rich and the Poor");
    AddSceneSplit(0x309, "A1_PainterHouse", false, "Tarsila's House");
    AddSceneSplit(0x30A, "A1_PainterExit1", false, "Backyard");
    AddSceneSplit(0x30B, "A1_PainterExit2", false, "Sewers");
    AddSceneSplit(0x30C, "A1_GameDesigner1", false, "The Big Walk");
    AddSceneSplit(0x30D, "A1_AvenueEast", false, "Main Avenue");
    AddSceneSplit(0x30E, "A1_Market", false, "Central Market");
    AddSceneSplit(0x30F, "A1_MusicianPath4", false, "Piano Alley");
    AddSceneSplit(0x310, "A1_AvenueEastTurn", false, "Corner's Club");
    AddSceneSplit(0x311, "A1_MinibossAvenue", false, "Beautiful Horizon Avenue");
    AddSceneSplit(0x312, "A1_MusicianHouse", false, "Thommaz's House");
    AddSceneSplit(0x313, "A1_GD2", false, "Theater Arena");
    AddSceneSplit(0x314, "A1_GD3", false, "One-way Street");
    AddSceneSplit(0x315, "A1_GD10", false, "Writer's Flow");
    AddSceneSplit(0x316, "A1_GD11", false, "Dancing Triplets");
    AddSceneSplit(0x317, "A1_GD12", false, "Mind Cross");
    AddSceneSplit(0x318, "A1_GD8", false, "The Dalvian Star");
    AddSceneSplit(0x319, "A1_GD4", false, "Buritis");
    AddSceneSplit(0x31A, "A1_GD13", false, "Corral Mountain Range");
    AddSceneSplit(0x31B, "A1_GD14", false, "Jonny B. Cave");
    AddSceneSplit(0x31C, "A1_GD15", false, "Concrete Cave");
    AddSceneSplit(0x31D, "A1_PreBoss", false, "Temple of Creation");
    AddSceneSplit(0x31E, "A1_BossFightRoom", false, "Temple of Creation");
    AddSceneSplit(0x31F, "A1_CreationStoneRoom", false, "Temple of Creation");
    AddSceneSplit(0x320, "A1_CityMainEntrance", false, "Main Gate");
    AddSceneSplit(0x321, "A1_BridgePath", false, "Village Outskirts");
    AddSceneSplit(0x322, "AB_Backtrack4", false, "Confusion Ruins");
    AddSceneSplit(0x323, "AB_Bridge", false, "The Colossal Bridge");
    AddSceneSplit(0x324, "AB_ToFieldCamp1", false, "The Traveler's Corridor");
    AddSceneSplit(0x325, "AB_FieldCamp", false, "Capital Outskirts");
    AddSceneSplit(0x326, "AB_ToDesert1", false, "Forest of Encoders");
    AddSceneSplit(0x327, "AB_ToDesert2", false, "Hoarders Woods");
    AddSceneSplit(0x328, "AB_ToDesert3", false, "Reclamation Grove");
    AddSceneSplit(0x329, "A2_Boss", false, "Overcast Gate Ruins");
    AddSceneSplit(0x32A, "A2_DesertJumper", false, "Corridor of Conscious Recollections");
    AddSceneSplit(0x32B, "A2_DesertCamp", false, "View of Solitude");
    AddSceneSplit(0x32C, "A2_DesertCrawler1", false, "The Wanderer Archives");
    AddSceneSplit(0x32D, "A2_Lotus", false, "The Drifter Archives");
    AddSceneSplit(0x32E, "A2_RBPlatformTutorial", false, "Eldarian Army Outpost");
    AddSceneSplit(0x32F, "A2_LotusCorridor", false, "Main Warehouse");
    AddSceneSplit(0x330, "A2_AnteChamberUpRight", false, "Archive of the Regretful");
    AddSceneSplit(0x331, "A2_ToInsideShortcut", false, "Museum Main Hall");
    AddSceneSplit(0x332, "A2_ToInside3", false, "Auditorium");
    AddSceneSplit(0x333, "A2_ToInside4", false, "Overburden Deposits");
    AddSceneSplit(0x334, "A2_ToInside5", false, "Museum Side Entrance");
    AddSceneSplit(0x335, "A2_Threeway", false, "C for Curiosity and Creation");
    AddSceneSplit(0x336, "A2_DesertAdvancedCamp", false, "Kid's Lecture House");
    AddSceneSplit(0x337, "A2_BeforeWriter", false, "Tower Stairway");
    AddSceneSplit(0x338, "A2_WriterRoom", false, "The Tower");
    AddSceneSplit(0x339, "A2_BarrierMiniboss", false, "Writer's Lone Balcony");
    AddSceneSplit(0x33A, "A2_RemembranceStoneAltar", false, "Remembrance Cliff");
    AddSceneSplit(0x33B, "A2_ToOutside1", false, "F for Findings and Fun");
    AddSceneSplit(0x33C, "A2_RemembranceWeaponRoom", false, "Quarter of a Distant Love");
    AddSceneSplit(0x33D, "A2_DesertCrawler3", false, "B for Bruises and Betweens");
    AddSceneSplit(0x33E, "A2_ToOutside2", false, "Corner of the Unasked Question");
    AddSceneSplit(0x33F, "A2_ToInside1", false, "S for Smiles and Scars");
    AddSceneSplit(0x340, "A2_ToRemembranceWeapon", false, "Remember Good Moments");
    AddSceneSplit(0x341, "AB_ToAF", false, "Corruption Transform");
    AddSceneSplit(0x342, "AF_4", false, "The Golden Threat");
    AddSceneSplit(0x343, "AF_5", false, "The Golden Pride");
    AddSceneSplit(0x344, "AF_FinalCamp", false, "Eldarian Gates");
    AddSceneSplit(0x345, "AF_6", false, "The Golden Honor");
    AddSceneSplit(0x346, "AF_7", false, "The Golden Menace");
    AddSceneSplit(0x347, "A3_Camp", false, "Reasoning Lock");
    AddSceneSplit(0x348, "A3_ToIntention1", false, "Logical Path");
    AddSceneSplit(0x349, "AF_8", false, "The Golden Walkway");
    AddSceneSplit(0x34A, "AF_9", false, "The Golden Storage");
    AddSceneSplit(0x34B, "A3_ToIntention2", false, "Intention Square");
    AddSceneSplit(0x34C, "A3_CribOfIntention", false, "Crib of Intention");
    AddSceneSplit(0x34D, "AB_Backtrack1", false, "Limits of Sanity");
    AddSceneSplit(0x34E, "AF_3", false, "The Golden Path");
    AddSceneSplit(0x34F, "AF_2", false, "The Golden Weaponry");
    AddSceneSplit(0x350, "AF_1", false, "The Breach in the Wall");
    AddSceneSplit(0x351, "A3_CityEntrance", false, "Capital Entrance");
    AddSceneSplit(0x352, "A3_Back7", false, "Side Turn");
    AddSceneSplit(0x353, "A4_Entrance", false, "View of the Deep");
    AddSceneSplit(0x354, "A4_Circle", false, "Dream Lands (Circle)");
    AddSceneSplit(0x355, "A4_DreamStone", false, "Dream Lands (Camp)");
    AddSceneSplit(0x356, "AF_12", false, "The Golden Angle");
    AddSceneSplit(0x357, "AF_MainRoom", false, "Path to Great Fortune");
    AddSceneSplit(0x358, "AF_D1", false, "ID Control");
    AddSceneSplit(0x359, "AF_D4", false, "Guardhouse");
    AddSceneSplit(0x35A, "AF_Boss", false, "Control Chamber");
}

init
{
    print("MemorySize: "+modules.First().ModuleMemorySize);
    if (modules.Any(m => m.ModuleName == "steamclient.dll")){
        version = "steam";        
    } else {
        version = "epic";
    }
    print("Version "+version);
}

gameTime
{
    if (!settings["remove_loading"]){
        return TimeSpan.FromSeconds(current.playedTime);
    }    
}

isLoading
{    
    return !settings["remove_loading"] || current.isTransitioning == 1;
}

start {
    if (current.playedTime > 0.0 && current.currentScene == "A1_Void4") {
        print("Start");        
        return true;
    } 
}

reset {
    if (current.playedTime < 1.0 && old.playedTime == 0.0 && current.currentScene == "A1_Void4") {
        print("Reset");
        return true;
    } 
}

update {
    if (current.currentScene == null) {
        return false;
    }

    vars.oldStoryEvents = vars.currentStoryEvents;
    vars.currentStoryEvents = new HashSet<int>(vars.oldStoryEvents);
    
    for (int i=0; i < current.numEvents; i++){
        int storyEvent = (int) memory.ReadPointer(((IntPtr)current.arrayEvents) + ((i*0x4)+0x10));        
        
        vars.currentStoryEvents.Add(storyEvent);
    }    

    vars.oldScenes = vars.currentScenes;
    if(!vars.currentScenes.Contains(current.currentScene)){
        vars.currentScenes = new HashSet<string>(vars.oldScenes);
        vars.currentScenes.Add(current.currentScene);
    }
}

split {
    var split = false;
    
    foreach (int storyEvent in vars.currentStoryEvents){
        if (!vars.oldStoryEvents.Contains(storyEvent)){
            print("Event occurred: "+vars.storyEventDictionary[storyEvent]);
        }
        split = split || (settings[vars.storyEventDictionary[storyEvent]] && !vars.oldStoryEvents.Contains(storyEvent));
    }
    
    if (current.currentScene != old.currentScene) {
        print("Current scene:"+current.currentScene);
        if(!vars.oldScenes.Contains(current.currentScene)){
            split = split || settings[current.currentScene];
        }
    }
    
    if (settings["split_current_scene"] && timer.CurrentSplit.Name.Contains(current.currentScene)) {
        split = true;
    }
    
    if (settings["split_saved_camp"] && timer.CurrentSplit.Name == current.baseCampScene) {
        split = true;
    }

    return split;
}
