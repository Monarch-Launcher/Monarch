import * as React from 'react';
import { invoke } from '@tauri-apps/api';
import type { MonarchGame, Result } from '../types';

type SearchGamesContextType = {
  searchedGames: MonarchGame[];
  searchGames: (searchString: string) => Promise<void>;
  error: boolean;
  loading: boolean;
  results: Result | undefined;
};

const initialState: SearchGamesContextType = {
  searchedGames: [],
  searchGames: async () => {},
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

  const searchGames = React.useCallback(async (searchString: string) => {
    try {
      setLoading(true);
      setError(false);
      const result: MonarchGame[] = await invoke('search_games', {
        name: searchString,
      });
      setResults({
        empty: result.length === 0,
        message: `Couldn't find any games for "${searchString}".`,
        searchString,
      });
      setSearchedGames([...result]);
    } catch (err) {
      setError(true);
    } finally {
      setLoading(false);
    }
  }, []);

  const value = React.useMemo(() => {
    return {
      searchedGames,
      searchGames,
      error,
      loading,
      results,
    };
  }, [searchedGames, searchGames, error, loading, results]);

  return (
    <SearchGamesContext.Provider value={value}>
      {children}
    </SearchGamesContext.Provider>
  );
};

export default SearchGamesProvider;
