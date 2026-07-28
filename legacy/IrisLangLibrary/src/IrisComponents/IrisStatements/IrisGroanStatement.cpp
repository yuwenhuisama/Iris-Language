#include "IrisComponents/IrisStatements/IrisGroanStatement.h"
#include "IrisComponents/IrisExpressions/IrisExpression.h"
#include "IrisInstructorMaker.h"
#include "IrisCompiler.h"
#include "IrisValidator/IrisExpressionValidateVisitor.h"


bool IrisGroanStatement::Generate()
{
	IrisCompiler* pCompiler = IrisCompiler::CurrentCompiler();
	IrisInstructorMaker* pMaker = IrisInstructorMaker::CurrentInstructor();
	pCompiler->SetLineNumber(m_nLineNumber);

	if(!m_pGroanExpression->Generate()) {
		return false;
	}

	pMaker->grn();

	return true;
}

IrisGroanStatement::IrisGroanStatement(IrisExpression* pGroanExpression) : m_pGroanExpression(pGroanExpression)
{
}


IrisGroanStatement::~IrisGroanStatement()
{
	if (m_pGroanExpression)
		delete m_pGroanExpression;

}

bool IrisGroanStatement::Validate()
{
	IrisExpressionValidateVisitor ievvExpressionVisitor;

	if (m_pGroanExpression && !m_pGroanExpression->Accept(&ievvExpressionVisitor)) {
		return false;
	}

	return true;
}
