import * as React from 'react';
import styled from 'styled-components';

const CardContainer = styled.div`
  display: inline-block;
  width: 15rem;
  height: 20rem;
  background-color: red;
  border-radius: 0.5rem;
  margin: 0.5rem;
`;

const GameCard = () => {
  return (
    <CardContainer>
      <p>This is a gamecard</p>
    </CardContainer>
  );
};

export default GameCard;
