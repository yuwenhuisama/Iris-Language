#include "IrisValidator/IrisStatementValidateVisitor.h"
#include "IrisComponents/IrisStatements/IrisStatement.h"



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
