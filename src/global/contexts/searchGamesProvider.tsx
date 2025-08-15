import { invoke } from '@tauri-apps/api/core';
import * as React from 'react';

import type { MonarchGame, Result } from '../types';

type SearchGamesContextType = {
  searchedGames: MonarchGame[];
  searchGames: (searchString: string, useMonarchCom: boolean) => Promise<void>;
  clearSearchResults: () => void;
  error: boolean;
  loading: boolean;
  results: Result | undefined;
};

const initialState: SearchGamesContextType = {
  searchedGames: [],
  searchGames: async () => { },
  clearSearchResults: () => { },
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
  const [error, setError] = React.useState(false);
  const [loading, setLoading] = React.useState(false);
  const [results, setResults] = React.useState<Result>();

  const searchGames = React.useCallback(async (searchString: string, useMonarchCom: boolean) => {
    try {
      setLoading(true);
      setError(false);
      const result: MonarchGame[] = await invoke('search_games', {
        name: searchString,
        useMonarch: useMonarchCom,
      });
      setResults({
        empty: result.length === 0,
        emptyMessage: `Couldn't find any games for "${searchString}".`,
        searchString,
      });
      setSearchedGames([...result]);
    } catch (err) {
      setError(true);
    } finally {
      setLoading(false);
    }
  }, []);

  const clearSearchResults = React.useCallback(() => {
    setSearchedGames([]);
    setResults(undefined);
    setError(false);
  }, []);

  const value = React.useMemo<SearchGamesContextType>(() => {
    return {
      searchedGames,
      searchGames,
      clearSearchResults,
      error,
      loading,
      results,
    };
  }, [searchedGames, searchGames, clearSearchResults, error, loading, results]);

  return (
    <SearchGamesContext.Provider value={value}>
      {children}
    </SearchGamesContext.Provider>
  );
};

export default SearchGamesProvider;
