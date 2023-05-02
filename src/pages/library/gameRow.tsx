import { FaCheck } from '@global/icons';
import * as React from 'react';
import styled from 'styled-components';

const Row = styled.div`
  display: flex;
  flex-direction: row;
  justify-content: space-between;
  align-items: center;
  padding: 1rem;
  border: 0.2rem solid black;
  border-radius: 0.5rem;
  transition: ease 0.2s;

  &:hover {
    background-color: ${({ theme }) => theme.colors.primary};
    color: black;
    cursor: pointer;
  }

  &:focus {
    background-color: ${({ theme }) => theme.colors.primary};
    color: black;
  }
`;

const GameTitle = styled.p`
  margin: 0;
  max-width: 80%;
`;

export enum OperationEnum {
  REMOVE = 'Remove',
  ADD = 'Add',
}

type Props = {
  id: string;
  name: string;
  updateSelectedGames: (id: string, operation: OperationEnum) => void;
};

const GameRow = ({ id, name, updateSelectedGames }: Props) => {
  const [gameSelected, setGameSelected] = React.useState(false);

  const toggleGameSelected = React.useCallback(() => {
    // If game is already selected, remove it; else add it
    if (gameSelected) {
      updateSelectedGames(id, OperationEnum.REMOVE);
    } else {
      updateSelectedGames(id, OperationEnum.ADD);
    }
    // Toggle state
    setGameSelected((prev) => !prev);
  }, [updateSelectedGames, id, gameSelected]);

  return (
    <Row onClick={toggleGameSelected}>
      <GameTitle>{name}</GameTitle>
      {gameSelected && <FaCheck />}
    </Row>
  );
};

export default GameRow;
