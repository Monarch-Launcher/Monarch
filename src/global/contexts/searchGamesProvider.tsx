import * as React from 'react';
import { invoke } from '@tauri-apps/api';
import type { MonarchGame } from '../types';

type MonarchGameContextType = {
  searchedGames: MonarchGame[];
  searchGames: (searchString: string) => Promise<void>;
  error: boolean;
  loading: boolean;
};

const initialState: MonarchGameContextType = {
  searchedGames: [],
  searchGames: async () => {},
  error: false,
  loading: false,
};

const SearchGamesContext =
  React.createContext<MonarchGameContextType>(initialState);
export const useSearchGames = () => React.useContext(SearchGamesContext);

type Props = {
  children: React.ReactNode;
};

const SearchGamesProvider = ({ children }: Props) => {
  const [searchedGames, setSearchedGames] = React.useState<MonarchGame[]>([]);
  const [error, setError] = React.useState(false);
  const [loading, setLoading] = React.useState(false);

  const searchGames = React.useCallback(async (searchString: string) => {
    try {
      setLoading(true);
      setError(false);
      const result: MonarchGame[] = await invoke('search_games', {
        name: searchString,
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
    };
  }, [searchedGames, searchGames, error, loading]);

  return (
    <SearchGamesContext.Provider value={value}>
      {children}
    </SearchGamesContext.Provider>
  );
};

export default SearchGamesProvider;
