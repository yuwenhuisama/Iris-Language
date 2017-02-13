#include "IrisComponents/IrisStatements/IrisStatement.h"
#include "IrisValidator/IrisAbstractStatementValidateVisitor.h"


IrisStatement::IrisStatement()
{
}

IrisStatement::~IrisStatement()
{
}

bool IrisStatement::Accept(IrisAbstractStatementValidateVisitor * pVisitor)
{
	return pVisitor->Visit(this);
}

bool IrisStatement::Validate()
{
	return true;
}