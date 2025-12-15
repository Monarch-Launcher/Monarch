import { invoke } from '@tauri-apps/api/core';
import * as React from 'react';

import type { MonarchGame, Result } from '../types';

type SearchGamesContextType = {
  searchedGames: MonarchGame[];
  searchGames: (
    searchString: string,
    sources: { monarch: boolean; steam: boolean; epic: boolean }
  ) => Promise<void>;
  clearSearchResults: () => void;
  error: boolean;
  loading: boolean;
  results: Result | undefined;
};

const initialState: SearchGamesContextType = {
  searchedGames: [],
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
  const [error, setError] = React.useState(false);
  const [loading, setLoading] = React.useState(false);
  const [results, setResults] = React.useState<Result>();

  const searchGames = React.useCallback(
    async (
      searchString: string,
      sources: { monarch: boolean; steam: boolean; epic: boolean },
    ) => {
      try {
        setLoading(true);
        setError(false);

        const promises: Promise<MonarchGame[]>[] = [];

      if (sources.monarch) {
        promises.push(invoke('search_games', {
          name: searchString,
          useMonarch: true,
        }));
      }

      if (sources.steam) {
        promises.push(invoke('search_games', {
          name: searchString,
          useMonarch: false,
        }));
      }

      // Epic is currently not supported by the backend, so we don't invoke anything for it yet.

      const resultsArray = await Promise.all(promises);
      const result = resultsArray.flat();

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
