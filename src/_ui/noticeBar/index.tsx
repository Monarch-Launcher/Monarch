import styled from 'styled-components';

export const NoticeBar = styled.div`
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
  padding: 0.75rem 1.5rem 0.75rem 1rem;
  margin: 0 1rem 1rem 1rem;
  border-radius: 0.5rem;
  background: rgba(255, 193, 7, 0.1);
  border: 1px solid rgba(255, 193, 7, 0.35);
`;

export const NoticeText = styled.p`
  margin: 0;
  font-size: 0.95rem;
`;
