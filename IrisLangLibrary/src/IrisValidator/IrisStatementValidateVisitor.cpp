#include "IrisValidator/IrisStatementValidateVisitor.h"
#include "IrisComponents/IrisStatements/IrisStatement.h"
#include "IrisUnil/IrisBlock.h"



IrisStatementValidateVisitor::IrisStatementValidateVisitor()
{
}


IrisStatementValidateVisitor::~IrisStatementValidateVisitor()
{
}

bool IrisStatementValidateVisitor::Visit(IrisStatement * pStatement)
{
	return pStatement->Validate();
}

bool IrisStatementValidateVisitor::Visit(IrisBlock * pBlock)
{
	return pBlock->Validate();
}
