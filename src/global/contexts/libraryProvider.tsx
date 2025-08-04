import { invoke } from '@tauri-apps/api/core';
import * as React from 'react';

import type { MonarchGame, Result } from '../types';

type LibraryContextType = {
  library: MonarchGame[];
  refreshLibrary: () => Promise<void>;
  error: boolean;
  loading: boolean;
  results: Result | undefined;
};

const initialState: LibraryContextType = {
  library: [],
  refreshLibrary: async () => { },
  error: false,
  loading: false,
  results: undefined,
};

const LibraryContext = React.createContext<LibraryContextType>(initialState);
export const useLibrary = () => React.useContext(LibraryContext);

type Props = {
  children: React.ReactNode;
};

const LibraryProvider = ({ children }: Props) => {
  const [library, setLibrary] = React.useState<MonarchGame[]>([]);
  const [error, setError] = React.useState(false);
  const [loading, setLoading] = React.useState(false);
  const [results, setResults] = React.useState<Result>();

  const refreshLibrary = React.useCallback(async () => {
    try {
      setError(false);
      setLoading(true);
      const result: MonarchGame[] = await invoke('refresh_library');
      setResults({
        empty: result.length === 0,
        emptyMessage:
          "Couldn't find any games on your system. Try adding a custom folder.",
      });
      setLibrary([...result]);
    } catch (err) {
      setError(true);
    } finally {
      setLoading(false);
    }
  }, []);

  const getLibrary = React.useCallback(async () => {
    try {
      setError(false);
      setLoading(true);
      const result: MonarchGame[] = await invoke('get_library');
      setLibrary([...result]);
    } catch (err) {
      setError(true);
    } finally {
      setLoading(false);
    }
  }, []);

  React.useEffect(() => {
    getLibrary();
  }, [getLibrary]);

  const value = React.useMemo<LibraryContextType>(() => {
    return {
      library,
      refreshLibrary,
      error,
      loading,
      results,
    };
  }, [library, refreshLibrary, error, loading, results]);

  return (
    <LibraryContext.Provider value={value}>{children}</LibraryContext.Provider>
  );
};

export default LibraryProvider;
