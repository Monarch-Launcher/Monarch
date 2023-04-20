import * as React from 'react';
import { invoke } from '@tauri-apps/api';
import type { MonarchGame } from '../types';

type LibraryContextType = {
  library: MonarchGame[];
  refreshLibrary: () => Promise<void>;
  error: boolean;
  loading: boolean;
};

const initialState: LibraryContextType = {
  library: [],
  refreshLibrary: async () => {},
  error: false,
  loading: false,
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

  const refreshLibrary = React.useCallback(async () => {
    try {
      setError(false);
      setLoading(true);
      const result: MonarchGame[] = await invoke('refresh_library');
      setLibrary([...result]);
    } catch (err) {
      setError(true);
    } finally {
      setLoading(false);
    }
  }, []);

  const value = React.useMemo(() => {
    return {
      library,
      refreshLibrary,
      error,
      loading,
    };
  }, [library, refreshLibrary, error, loading]);

  return (
    <LibraryContext.Provider value={value}>{children}</LibraryContext.Provider>
  );
};

export default LibraryProvider;
