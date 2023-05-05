import { invoke } from '@tauri-apps/api';
import * as React from 'react';

import type { Collection, MonarchGame, Result } from '../types';

type LibraryContextType = {
  library: MonarchGame[];
  refreshLibrary: () => Promise<void>;
  error: boolean;
  loading: boolean;
  results: Result | undefined;
  collections: Collection[];
};

const initialState: LibraryContextType = {
  library: [],
  refreshLibrary: async () => {},
  error: false,
  loading: false,
  results: undefined,
  collections: [],
};

const LibraryContext = React.createContext<LibraryContextType>(initialState);
export const useLibrary = () => React.useContext(LibraryContext);

type Props = {
  children: React.ReactNode;
};

// TODO: remove this
const mockCollections: Collection[] = [
  {
    id: 'some kind of id',
    name: 'cool games',
    gameIds: [
      '10006750510124000270',
      '12745051691570522837',
      '1947104710968256949',
      '14536788471735206296',
    ],
  },
  {
    id: 'another id',
    name: 'games with "ark"',
    gameIds: [
      '15098186198963317337',
      '14747636517855909739',
      '9667814351563258295',
      '8826081208144110070',
      '2930480368731506396',
    ],
  },
];

const LibraryProvider = ({ children }: Props) => {
  const [library, setLibrary] = React.useState<MonarchGame[]>([]);
  const [error, setError] = React.useState(false);
  const [loading, setLoading] = React.useState(false);
  const [results, setResults] = React.useState<Result>();

  // TODO: Change mockCollections to empty array
  const [collections, setCollections] =
    React.useState<Collection[]>(mockCollections);

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

  // TODO: remove these comments
  // @ts-ignore
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  const getCollections = React.useCallback(async () => {
    try {
      const result: Collection[] = await invoke('get_collections');
      setCollections([...result]);
    } catch (err) {
      // TODO: proper error and loading state
    }
  }, []);

  React.useEffect(() => {
    getLibrary();
    // getCollections();
  }, [getLibrary]);

  const value = React.useMemo<LibraryContextType>(() => {
    return {
      library,
      refreshLibrary,
      error,
      loading,
      results,
      collections,
    };
  }, [library, refreshLibrary, error, loading, results, collections]);

  return (
    <LibraryContext.Provider value={value}>{children}</LibraryContext.Provider>
  );
};

export default LibraryProvider;
