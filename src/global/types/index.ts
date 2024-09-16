export type MonarchGame = {
  id: string;
  platform_id: string;
  executable_path: string;
  name: string;
  platform: string;
  thumbnail_path: string;
  store_page: string;
};

export type Result = {
  empty: boolean;
  emptyMessage: string;
  searchString?: string;
};

export type Collection = {
  id: string;
  name: string;
  gameIds: string[];
};

// TODO: Let Dre make this better...
export type MonarchSettings = {
  monarch: MonarchSetting;
  quicklaunch: QuicklaunchSetting;
  steam: SteamSetting;
  epic: EpicSetting;
};

export type MonarchSetting = {
  monarch_home: string;
  send_logs: boolean;
  run_on_startup: boolean;
  start_minimized: boolean;
  game_folder: string;
}

export type QuicklaunchSetting = {
  enabled: boolean;
  open_shortcut: string;
  close_shortcut: string;
  size: string;
}

export type SteamSetting = {
  game_folders: string;
  manage: boolean;
  username: string;
}

export type EpicSetting = {
  game_folders: string;
  manage: boolean;
  username: string;
}