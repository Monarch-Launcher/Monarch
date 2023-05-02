import spinner from '@assets/spinner.svg';
import styled from 'styled-components';

const StyledSpinner = styled.img`
  width: 100px;
`;

const Spinner = () => {
  return <StyledSpinner src={spinner} alt="loading-spinner" />;
};

export default Spinner;
