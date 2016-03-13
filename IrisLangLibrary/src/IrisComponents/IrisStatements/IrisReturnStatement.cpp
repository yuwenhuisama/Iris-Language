#include "IrisComponents/IrisStatements/IrisReturnStatement.h"
#include "IrisComponents/IrisExpressions/IrisExpression.h"
#include "IrisInstructorMaker.h"
#include "IrisComponents/IrisVirtualCodeStructures.h"
#include "IrisCompiler.h"
#include "IrisFatalErrorHandler.h"

IrisReturnStatement::IrisReturnStatement(IrisExpression* pReturnExpression) : m_pReturnExpression(pReturnExpression)
{
}


bool IrisReturnStatement::Generate()
{
	IrisInstructorMaker* pMaker = IrisInstructorMaker::CurrentInstructor();
	IrisCompiler* pCompiler = IrisCompiler::CurrentCompiler();
	pCompiler->SetLineNumber(m_nLineNumber);

	if (!pCompiler->UpperWithMethod()) {
		IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::ContinueIrregular, m_nLineNumber, pCompiler->GetCurrentFileIndex(), "Return Statement CAN ONLY be used in method.");
		return false;
	}

	if (m_pReturnExpression) {
		m_pReturnExpression->Generate();
	}
	else {
		pMaker->load_nil();
	}
	pMaker->rtn();
	return true;
}

IrisReturnStatement::~IrisReturnStatement()
{
	if (m_pReturnExpression)
		delete m_pReturnExpression;

}
