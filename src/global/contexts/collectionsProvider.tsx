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

const CollectionsProvider = ({ children }: Props) => {
  const [collections, setCollections] = React.useState<Collection[]>([]);
  const [error, setError] = React.useState(false);
  const [loading, setLoading] = React.useState(false);

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

  const createCollection = React.useCallback(
    async (collectionName: string, gameIds: string[]) => {
      try {
        setError(false);
        setLoading(true);
        await invoke('create_collection', { collectionName, gameIds });
        await getCollections();
      } catch (err) {
        setError(true);
      } finally {
        setLoading(false);
      }
    },
    [getCollections],
  );

  const updateCollection = React.useCallback(
    async (id: string, newName: string, gameIds: string[]) => {
      try {
        setError(false);
        setLoading(true);
        await invoke('update_collection', { id, newName, gameIds });
        await getCollections();
      } catch (err) {
        setError(true);
      } finally {
        setLoading(false);
      }
    },
    [getCollections],
  );

  const deleteCollection = React.useCallback(
    async (id: string) => {
      try {
        setError(false);
        setLoading(true);
        await invoke('delete_collection', { id });
        await getCollections();
      } catch (err) {
        setError(true);
      } finally {
        setLoading(false);
      }
    },
    [getCollections],
  );

  React.useEffect(() => {
    getCollections();
  }, [getCollections]);

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
