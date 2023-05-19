import { invoke } from '@tauri-apps/api';
import * as React from 'react';

import type { Collection } from '../types';

type CollectionsContextType = {
  collections: Collection[];
  error: boolean;
  loading: boolean;
  updateCollection: (
    id: string,
    newName: string,
    gameIds: string[],
  ) => Promise<void>;
  deleteCollection: (id: string) => Promise<void>;
  createCollection: (
    collectionName: string,
    gameIds: string[],
  ) => Promise<void>;
};

const initialState: CollectionsContextType = {
  collections: [],
  error: false,
  loading: false,
  updateCollection: async () => {},
  deleteCollection: async () => {},
  createCollection: async () => {},
};

const CollectionsContext =
  React.createContext<CollectionsContextType>(initialState);
export const useCollections = () => React.useContext(CollectionsContext);

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

const CollectionsProvider = ({ children }: Props) => {
  // TODO: Change mockCollections to empty array
  const [collections, setCollections] =
    React.useState<Collection[]>(mockCollections);
  const [error, setError] = React.useState(false);
  const [loading, setLoading] = React.useState(false);

  const createCollection = React.useCallback(
    async (collectionName: string, gameIds: string[]) => {
      try {
        setError(false);
        setLoading(true);
        await invoke('create_collection', { collectionName, gameIds });
      } catch (err) {
        setError(true);
      } finally {
        setLoading(false);
      }
    },
    [],
  );

  const updateCollection = React.useCallback(
    async (id: string, newName: string, gameIds: string[]) => {
      try {
        setError(false);
        setLoading(true);
        await invoke('update_collection', { id, newName, gameIds });
      } catch (err) {
        setError(true);
      } finally {
        setLoading(false);
      }
    },
    [],
  );

  const deleteCollection = React.useCallback(async (id: string) => {
    try {
      setError(false);
      setLoading(true);
      await invoke('delete_collection', { id });
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
      setLoading(true);
      setError(false);
      const result: Collection[] = await invoke('get_collections');
      setCollections([...result]);
    } catch (err) {
      setError(true);
    } finally {
      setLoading(false);
    }
  }, []);

  React.useEffect(() => {
    // getCollections();
  }, []);

  const value = React.useMemo<CollectionsContextType>(() => {
    return {
      collections,
      error,
      loading,
      updateCollection,
      deleteCollection,
      createCollection,
    };
  }, [
    collections,
    error,
    loading,
    updateCollection,
    deleteCollection,
    createCollection,
  ]);

  return (
    <CollectionsContext.Provider value={value}>
      {children}
    </CollectionsContext.Provider>
  );
};

export default CollectionsProvider;
