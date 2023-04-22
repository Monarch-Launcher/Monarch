export type MonarchGame = {
  id: number;
  executable_path: string;
  name: string;
  platform: string;
  thumbnail_path: string;
};

export type Result = {
  empty: boolean;
  message: string;
  searchString?: string;
};
