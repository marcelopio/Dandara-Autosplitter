state("Dandara")
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

startup {
    
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
    });
    
    vars.ResetVars();

    vars.timerResetVars = (EventHandler)((s, e) => {
        vars.ResetVars();
    });
    timer.OnStart += vars.timerResetVars;
    
    vars.storyEventDictionary = new Dictionary<int, string>();
    Action<int, string, bool, string> AddEventSplit = (key, name, active, description) => {
        settings.Add(name, active, name, "event");
        settings.SetToolTip(name, description);
        vars.storyEventDictionary[key] = name;
    };
    
    settings.Add("event", true, "event");
    settings.Add("remove_loading", false, "Remove Loading");
    settings.Add("split_current_scene", true, "Split on current scene name");
    settings.Add("split_saved_camp", false, "Split on current camp name, does not work with current scene enabled");
    
    AddEventSplit(0x1, "Started", false, "Game started");
    AddEventSplit(0x2, "Ended", true, "Killed Eldar 1 hit form");
    AddEventSplit(0x3, "OpenedEldarDoor", false, "");
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
    AddEventSplit(0x3A, "Miniboss_Kill_2", false, "");
    AddEventSplit(0x3B, "Miniboss_Kill_3", false, "");
    AddEventSplit(0x3C, "Miniboss_Kill_4", false, "");
    AddEventSplit(0x64, "PU_Stone_Creation", false, "Got stone of creation");
    AddEventSplit(0x65, "PU_Stone_Remembrance", true, "Got rock of Remembrance");
    AddEventSplit(0x66, "PU_Stone_Intention", true, "Got stone of Intention");
    AddEventSplit(0x67, "PU_Stone_Dreams", true, "Got pearl of dreams");
    AddEventSplit(0x6C, "PU_DLCF_FinalKey", false, "");
    AddEventSplit(0x6E, "PU_Health", false, "");
    AddEventSplit(0x6F, "PU_Ammo", false, "");
    AddEventSplit(0x70, "PU_HealthFlask", true, "Got first health flask");
    AddEventSplit(0x71, "PU_Map", false, "Got map");
    AddEventSplit(0x72, "PU_Shield", false, "");
    AddEventSplit(0x73, "PU_ManaFlask", false, "");
    AddEventSplit(0x74, "PU_HealthFlaskUpgrade", false, "Got first health flask");
    AddEventSplit(0x75, "PU_ManaFlaskUpgrade", false, "");
    AddEventSplit(0x76, "PU_Money", false, "");
    AddEventSplit(0x77, "PU_DandaraArrow", false, "");
    AddEventSplit(0x78, "PU_ManaWeapon", false, "");
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
    AddEventSplit(0xC8, "D_Painter", false, "Encountered the painter");
    AddEventSplit(0xC9, "D_Musician", false, "Encountered the musician");
    AddEventSplit(0xCA, "D_CreatGuardBridge", false, "");
    AddEventSplit(0xCB, "D_Eldar1", false, "");
    AddEventSplit(0xCC, "D_DLCF_MastersIntroduction", false, "");
    AddEventSplit(0xCD, "D_DLCF_ExplorerFinish", false, "");
    AddEventSplit(0xCE, "D_DLCF_NobleFinish", false, "");
    AddEventSplit(0xCF, "D_DLCF_PersistentFinish", false, "");
    AddEventSplit(0xD0, "D_DLCF_ExplorerStart", false, "");
    AddEventSplit(0xD1, "D_DLCF_NobleStart", false, "");
    AddEventSplit(0xD2, "D_DLCF_PersistentStart", false, "");
    AddEventSplit(0xFA, "HUD_Unlock", false, "");
    AddEventSplit(0xFB, "HUD_TutorialClear", false, "");
    AddEventSplit(0x12C, "Char_Painter", false, "");
    AddEventSplit(0x12D, "Char_Musician", false, "");
    AddEventSplit(0x12E, "Char_GameDev", false, "");
    AddEventSplit(0x12F, "Char_Boss1_NoName", false, "");
    AddEventSplit(0x130, "Char_A1Boss", false, "Entered Augustus room");
    AddEventSplit(0x131, "Char_A2Boss", false, "");
    AddEventSplit(0x132, "Char_Writer", false, "");
    AddEventSplit(0x133, "Char_AFBoss1", false, "");
    AddEventSplit(0x134, "Char_AFBoss2", false, "");
    AddEventSplit(0x135, "Char_HeartCourage", false, "");
    AddEventSplit(0x136, "Char_DLCF_Boss", false, "");
    AddEventSplit(0x137, "Char_DLCF_Explorer", false, "");
    AddEventSplit(0x138, "Char_DLCF_Noble", false, "");
    AddEventSplit(0x139, "Char_DLCF_Persistent", false, "");
    AddEventSplit(0x13A, "Char_Lazuli", false, "");
    AddEventSplit(0x13B, "Char_Lazuli_NoName", false, "");
    AddEventSplit(0x13C, "Char_DLCF_Nara", false, "");
    AddEventSplit(0x13D, "Char_DLCF_ElderDandara", false, "");
    AddEventSplit(0x1F4, "Cutscene_FavelaHUB", false, "");
    AddEventSplit(0x1F5, "Cutscene_CampDiscovery", false, "");
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
    if (current.playedTime > 0.0 && old.playedTime == 0.0) {
        print("Start");        
        return true;
    } 
}

reset {
    if (old.playedTime > 0.0 && current.playedTime == 0.0) {
        print("Reset");
        return true;
    } 
}

update {
    vars.oldStoryEvents = vars.currentStoryEvents;
    vars.currentStoryEvents = new HashSet<int>(vars.oldStoryEvents);
    
    for (int i=0; i < current.numEvents; i++){
        int storyEvent = (int) memory.ReadPointer(((IntPtr)current.arrayEvents) + ((i*0x4)+0x10));        
        
        vars.currentStoryEvents.Add(storyEvent);
    }    
}

split {
    var split = false;
    
    foreach (int storyEvent in vars.currentStoryEvents){
        split = split || (settings[vars.storyEventDictionary[storyEvent]] && !vars.oldStoryEvents.Contains(storyEvent));
    }
    
    if (settings["split_current_scene"] && timer.CurrentSplit.Name == current.currentScene) {
        split = true;
    }
    
    if (settings["split_saved_camp"] && timer.CurrentSplit.Name == current.baseCampScene) {
        split = true;
    }

    return split;
}
