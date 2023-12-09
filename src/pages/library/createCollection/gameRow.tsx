import { FaCheck } from '@global/icons';
import * as React from 'react';
import styled from 'styled-components';

const Row = styled.div`
  display: flex;
  flex-direction: row;
  justify-content: space-between;
  align-items: center;
  padding: 1rem;
  border: 0.2rem solid ${({ theme }) => theme.colors.secondary};
  border-radius: 0.5rem;
  transition: ease 0.2s;
  background-color: ${({ theme }) => theme.colors.secondary};
  color: ${({ theme }) => theme.colors.primary};

  > svg {
    fill: ${({ theme }) => theme.colors.lightgreen};
  }

  &:hover,
  &:focus {
    background-color: ${({ theme }) => theme.colors.primary};
    border-color: ${({ theme }) => theme.colors.primary};
    color: ${({ theme }) => theme.colors.secondary};
    cursor: pointer;

    > svg {
      fill: ${({ theme }) => theme.colors.darkgreen};
    }
  }
`;

const GameTitle = styled.p`
  margin: 0;
  max-width: 80%;
  font-weight: 600;
`;

export enum OperationEnum {
  REMOVE = 'Remove',
  ADD = 'Add',
}

type Props = {
  id: string;
  name: string;
  isSelected?: boolean;
  updateSelectedGames: (id: string, operation: OperationEnum) => void;
};

const GameRow = ({ id, name, isSelected, updateSelectedGames }: Props) => {
  const [gameSelected, setGameSelected] = React.useState(isSelected);

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
