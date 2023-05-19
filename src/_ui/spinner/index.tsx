import Spinner from '@assets/spinner.svg';
import styled, { AnyStyledComponent } from 'styled-components';

const StyledSpinner = styled(Spinner as AnyStyledComponent)`
  width: 100px;
`;

export default () => {
  return <StyledSpinner />;
};
