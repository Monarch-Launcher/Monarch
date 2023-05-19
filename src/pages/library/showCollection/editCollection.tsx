import Button from '@_ui/button';
import Modal from '@_ui/modal';
import SearchBar from '@_ui/searchBar';
import { useCollections } from '@global/contexts/collectionsProvider';
import { useLibrary } from '@global/contexts/libraryProvider';
import { BiCheck, BiLeftArrowAlt, BiTrash, MdClose } from '@global/icons';
import { Collection, MonarchGame } from '@global/types';
import { useDisclosure } from '@mantine/hooks';
import * as React from 'react';
import { useForm } from 'react-hook-form';
import styled from 'styled-components';

import GameRow from '../createCollection/gameRow';

const StyledInput = styled.input`
  background-color: ${({ theme }) => theme.colors.black};
  border-radius: 0.5rem;
  border: none;
  padding: 0.5rem;
  &:focus {
    outline: none;
  }
`;

const Section = styled.div`
  margin-bottom: 1rem;
`;

const Label = styled.p`
  margin: 0 0 0.5rem 0;
  font-size: 1.1rem;
  font-weight: 600;
`;

const GameContainer = styled.div`
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  margin-top: 1rem;
  height: 50vh;
  overflow-y: scroll;
`;

const ButtonContainer = styled.div<{ $justify: string }>`
  display: flex;
  align-items: center;
  justify-content: ${({ $justify }) => $justify};
  gap: 1rem;
  margin: 2rem 0 1rem;
`;

const Row = styled.div`
  display: flex;
  gap: 1rem;
`;

const ModalText = styled.p`
  margin: 0 0 2rem 0;
  text-align: center;
  font-size: 1.1rem;
`;

const ModalHeaderContainer = styled.div`
  display: flex;
  flex-direction: row;
  justify-content: space-between;
  align-items: flex-start;
  width: 100%;
`;

const ModalHeader = styled.h2`
  margin: 0.5rem 0;
  color: ${({ theme }) => theme.colors.primary};
`;

type Props = {
  collection: Collection;
  toggleEditing: () => void;
};

type FormValues = {
  newName: string;
  gameIds: string[];
};

const EditCollectionForm = ({ collection, toggleEditing }: Props) => {
  const [searchTerm, setSearchTerm] = React.useState('');
  const [opened, { open, close }] = useDisclosure(false);

  const { library } = useLibrary();
  const { updateCollection, deleteCollection } = useCollections();
  const { register, handleSubmit, setValue, getValues } = useForm<FormValues>();

  const modalHeader = React.useMemo<JSX.Element>(() => {
    return (
      <ModalHeaderContainer>
        <ModalHeader>
          Are you sure you want to delete &apos;{collection.name}
          &apos; ?
        </ModalHeader>
        <Button type="button" variant="icon" onClick={close}>
          <MdClose color="black" size={24} />
        </Button>
      </ModalHeaderContainer>
    );
  }, [collection.name, close]);

  const filteredLibrary = React.useMemo<MonarchGame[]>(() => {
    return library.filter((game) =>
      game.name
        .replace(/[.,/#!$%^&*;:{}=\-_`~()]/g, '')
        .toLowerCase()
        .match(searchTerm.toLowerCase()),
    );
  }, [library, searchTerm]);

  const handleSearchtermChange = React.useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      setSearchTerm(e.target.value);
    },
    [],
  );

  const handleDelete = React.useCallback(async () => {
    await deleteCollection(collection.id);
  }, [deleteCollection, collection.id]);

  const onSubmit = React.useCallback(
    async (formvalues: FormValues) => {
      // TODO: Validation checks
      await updateCollection(
        collection.id,
        formvalues.newName,
        formvalues.gameIds,
      );
    },
    [collection.id, updateCollection],
  );

  const handleUpdateGameIds = React.useCallback(
    (gameId: string) => {
      const currentGameIds = getValues('gameIds');
      // Game is already in, remove game
      if (currentGameIds.includes(gameId)) {
        const indexToRemove = currentGameIds.indexOf(gameId);
        currentGameIds.splice(indexToRemove, 1);
        setValue('gameIds', [...currentGameIds]);
        return;
      }
      setValue('gameIds', [...currentGameIds, gameId]);
    },
    [setValue, getValues],
  );

  React.useEffect(() => {
    setValue('gameIds', collection.gameIds);
  }, [setValue, collection.gameIds]);

  return (
    <form onSubmit={handleSubmit(onSubmit)}>
      <Section>
        <Label>Collection name</Label>
        <StyledInput
          placeholder="New name"
          defaultValue={collection.name}
          {...register('newName', { minLength: 1 })}
        />
      </Section>
      <Section>
        <Label>Select games</Label>
        <SearchBar
          value={searchTerm}
          onChange={handleSearchtermChange}
          placeholder="Search..."
        />
        <GameContainer>
          {filteredLibrary.map((game) => (
            <GameRow
              key={game.id}
              id={game.id}
              name={game.name}
              isSelected={collection.gameIds.includes(game.id)}
              updateSelectedGames={() => {
                handleUpdateGameIds(game.id);
              }}
            />
          ))}
        </GameContainer>
      </Section>
      <ButtonContainer $justify="space-between">
        <Button
          type="button"
          variant="danger"
          rightIcon={BiTrash}
          onClick={open}
        >
          Delete
        </Button>
        <Row>
          <Button
            type="button"
            variant="secondary"
            leftIcon={BiLeftArrowAlt}
            onClick={toggleEditing}
          >
            Back
          </Button>
          <Button type="submit" variant="primary" rightIcon={BiCheck}>
            Save
          </Button>
        </Row>
      </ButtonContainer>
      <Modal
        opened={opened}
        onClose={close}
        title={modalHeader}
        zIndex={400}
        centered
        withCloseButton={false}
        size="50vw"
      >
        <ModalText>⚠️ This action cannot be undone ⚠️</ModalText>
        <ButtonContainer $justify="right">
          <Button
            type="button"
            variant="primary"
            leftIcon={MdClose}
            onClick={close}
          >
            Cancel
          </Button>
          <Button
            type="button"
            variant="danger"
            rightIcon={BiTrash}
            onClick={handleDelete}
          >
            Delete
          </Button>
        </ButtonContainer>
      </Modal>
    </form>
  );
};

export default EditCollectionForm;
