export type MonarchGame = {
  id: string;
  platform_id: string;
  executable_path: string;
  name: string;
  platform: string;
  thumbnail_path: string;
  store_page: string;
  compatibility: string;
  launch_args: string;
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

type LauncherType = {
  game_folders: string[];
  manage: boolean;
  username: string;
  twofa: boolean;
};

export type Settings = {
  epic: LauncherType;
  steam: LauncherType;
  monarch: {
    game_folder: string;
    monarch_home: string;
    run_on_startup: boolean;
    send_logs: boolean;
    start_minimized: boolean;
  };
  quicklaunch: {
    close_shortcut: string;
    open_shortcut: string;
    enabled: boolean;
    size: string;
  };
};

export type ProtonVersion = {
  name: string;
  path: string;
};
