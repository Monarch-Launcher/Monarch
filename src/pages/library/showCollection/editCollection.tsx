import Button from '@_ui/button';
import { useLibrary } from '@global/contexts/libraryProvider';
import { Collection, MonarchGame } from '@global/types';
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
  font-weight: 500;
`;

const GameContainer = styled.div`
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  margin-top: 1rem;
  height: 50vh;
  overflow-y: scroll;
`;

type Props = {
  collection: Collection;
  toggleEditing: () => void;
};

type FormValues = {
  newName: string;
  games: string[];
};

const EditCollectionForm = ({ collection, toggleEditing }: Props) => {
  const [searchTerm, setSearchTerm] = React.useState('');
  const { library } = useLibrary();
  const { register, handleSubmit } = useForm<FormValues>();

  const onSubmit = React.useCallback(() => {}, []);

  const filteredLibrary = React.useMemo<MonarchGame[]>(() => {
    return library.filter((game) =>
      game.name
        .replace(/[.,/#!$%^&*;:{}=\-_`~()]/g, '')
        .toLowerCase()
        .match(searchTerm.toLowerCase()),
    );
  }, [library, searchTerm]);

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
        <GameContainer>
          {filteredLibrary.map((game) => (
            <GameRow
              key={game.id}
              id={game.id}
              name={game.name}
              updateSelectedGames={() => {}}
            />
          ))}
        </GameContainer>
      </Section>
      <Button type="button" variant="primary" onClick={toggleEditing}>
        Back
      </Button>
      <Button type="button" variant="primary" onClick={toggleEditing}>
        Save
      </Button>
    </form>
  );
};

export default EditCollectionForm;
