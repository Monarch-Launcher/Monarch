import error from '@assets/error.svg';
import styled from 'styled-components';

import Button from '../button';

const ErrorContainer = styled.div`
  display: flex;
  flex-direction: column;
  align-items: center;
  max-width: 18.75rem;
  gap: 1rem;
  margin: 0 auto;
`;

const ErrorSvg = styled.img`
  width: 150px;
  align-self: center;
`;

const Title = styled.h3`
  margin: 0;
  align-self: center;
`;

const Description = styled.p`
  margin: 0;
`;

const RetryButton = styled(Button)`
  justify-content: center;
`;

type Props = {
  description: string;
  onRetry?: () => void;
};

const Error = ({ description, onRetry }: Props) => {
  return (
    <ErrorContainer>
      <ErrorSvg src={error} alt="error-sign" />
      <Title>Oops! Something went wrong</Title>
      <Description>{description}</Description>
      {onRetry && (
        <RetryButton
          type="button"
          variant="primary"
          onClick={onRetry}
          fullWidth
        >
          Retry
        </RetryButton>
      )}
    </ErrorContainer>
  );
};

export default Error;
