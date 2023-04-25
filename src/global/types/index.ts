export type MonarchGame = {
  id: string;
  platform_id: string;
  executable_path: string;
  name: string;
  platform: string;
  thumbnail_path: string;
};

export type Result = {
  empty: boolean;
  emptyMessage: string;
  searchString?: string;
};
