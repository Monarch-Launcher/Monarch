import { invoke } from '@tauri-apps/api/core';
import * as React from 'react';

import type { MonarchGame, MonarchWebApiGame, Result, SearchFilter } from '../types';

type SearchGamesContextType = {
  searchedGames: MonarchGame[];
  webApiGames: MonarchWebApiGame[];
  searchGames: (
    searchString: string,
    filter: SearchFilter
  ) => Promise<void>;
  clearSearchResults: () => void;
  error: boolean;
  loading: boolean;
  results: Result | undefined;
};

const initialState: SearchGamesContextType = {
  searchedGames: [],
  webApiGames: [],
  searchGames: async () => {},
  clearSearchResults: () => {},
  error: false,
  loading: false,
  results: undefined,
};

const SearchGamesContext =
  React.createContext<SearchGamesContextType>(initialState);
export const useSearchGames = () => React.useContext(SearchGamesContext);

type Props = {
  children: React.ReactNode;
};

const SearchGamesProvider = ({ children }: Props) => {
  const [searchedGames, setSearchedGames] = React.useState<MonarchGame[]>([]);
  const [webApiGames, setWebApiGames] = React.useState<MonarchWebApiGame[]>([]);
  const [error, setError] = React.useState(false);
  const [loading, setLoading] = React.useState(false);
  const [results, setResults] = React.useState<Result>();

  const searchGames = React.useCallback(
    async (
      searchString: string,
      filter: SearchFilter,
    ) => {
      try {
        setLoading(true);
        setError(false);

        // Backend now returns MonarchWebApiGame[]
        const apiGamesResponse = await invoke<MonarchWebApiGame[]>('search_games', {
          name: searchString,
          filter,
        });

        // Convert MonarchWebApiGame[] to MonarchGame[] for compatibility
        const convertedGames: MonarchGame[] = apiGamesResponse.map((webGame) => ({
          id: webGame.id,
          platform_id: webGame.platforms[0]?.platform_id || '',
          executable_path: '',
          name: webGame.name,
          platform: webGame.platforms[0]?.name || '',
          thumbnail_path: webGame.thumbnail_path,
          thumbnail_url: webGame.cover_url,
          store_page: webGame.platforms[0]?.store_page || '',
          compatibility: '',
          launch_args: '',
          install_dir: '',
          description: webGame.summary,
        }));

        setResults({
          empty: convertedGames.length === 0,
          emptyMessage: `Couldn't find any games for "${searchString}".`,
          searchString,
        });
        setSearchedGames([...convertedGames]);
        setWebApiGames([...apiGamesResponse]);
      } catch (err) {
        setError(true);
      } finally {
        setLoading(false);
      }
    }, []);

  const clearSearchResults = React.useCallback(() => {
    setSearchedGames([]);
    setWebApiGames([]);
    setResults(undefined);
    setError(false);
  }, []);

  const value = React.useMemo<SearchGamesContextType>(() => {
    return {
      searchedGames,
      webApiGames,
      searchGames,
      clearSearchResults,
      error,
      loading,
      results,
    };
  }, [searchedGames, webApiGames, searchGames, clearSearchResults, error, loading, results]);

  return (
    <SearchGamesContext.Provider value={value}>
      {children}
    </SearchGamesContext.Provider>
  );
};

export default SearchGamesProvider;
