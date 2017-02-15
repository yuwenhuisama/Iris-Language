#include "IrisComponents/IrisExpressions/IrisFunctionCallExpression.h"
#include "IrisComponents/IrisExpressions/IrisExpression.h"
#include "IrisUnil/IrisIdentifier.h"
#include "IrisComponents/IrisParts/IrisClosureBlockLiteral.h"
#include "IrisCompiler.h"
#include "IrisInstructorMaker.h"
#include "IrisValidator/IrisExpressionValidateVisitor.h"
#include "IrisValidator/IrisStatementValidateVisitor.h"
#include "IrisFatalErrorHandler.h"

bool IrisFunctionCallExpression::Generate()
{
	IrisCompiler* pCompiler = IrisCompiler::CurrentCompiler();
	IrisInstructorMaker* pMaker = IrisInstructorMaker::CurrentInstructor();
	pCompiler->SetLineNumber(m_nLineNumber);

	unsigned int nIdentifierIndex = pCompiler->GetIdentifierIndex(m_pFunctionName->GetIdentifierString(), pCompiler->GetCurrentFileIndex());
	unsigned int nParameterCount = m_pParameters ? m_pParameters->GetSize() : 0;

	if(m_pClosureBlock){
		if(!m_pClosureBlock->Generate())
			return false;
	}

	// 逐个计算参数
	if (m_pParameters) {
		if(!m_pParameters->Ergodic(
			[&](IrisExpression*& pExpression) {
			if (!pExpression->Generate()) {
				return false;
			}
			pMaker->push();
			return true;
		}
		))
			return false;
	}

	if (m_pObject) {
		if (!m_pObject->Generate()) {
			return false;
		}
		pMaker->push_env();
		pMaker->cre_env();
		pMaker->nol_call(nIdentifierIndex, nParameterCount);
	}
	else{
		pMaker->push_env();
		pMaker->cre_env();
		pMaker->hid_call(nIdentifierIndex, nParameterCount);
	}
	pMaker->pop_env();
	if (nParameterCount > 0) {
		pMaker->pop(nParameterCount);
	}

	return true;
}

IrisFunctionCallExpression::IrisFunctionCallExpression(IrisExpression* pObject, IrisIdentifier* pFunctionName, IrisList<IrisExpression*>* pParameters, IrisClosureBlockLiteral* pClosureBlock) : m_pObject(pObject), m_pFunctionName(pFunctionName), m_pParameters(pParameters), m_pClosureBlock(pClosureBlock)
{
}


IrisFunctionCallExpression::~IrisFunctionCallExpression()
{
	if (m_pObject) {
		delete m_pObject;
	}
	if (m_pFunctionName) {
		delete m_pFunctionName;
	}
	if (m_pParameters) {
		m_pParameters->Ergodic(
			[](IrisExpression*& pExpression) -> bool {delete pExpression; pExpression = nullptr; return true; }
		);
		delete m_pParameters;
	}
	if (m_pClosureBlock) {
		delete m_pClosureBlock;
	}
}

bool IrisFunctionCallExpression::Validate()
{
	auto pCompiler = IrisCompiler::CurrentCompiler();
	IrisExpressionValidateVisitor ievvExpressionVisitor;
	IrisStatementValidateVisitor isvvStatementVisitor;

	if (m_pObject && !m_pObject->Accept(&ievvExpressionVisitor)) {
		return false;
	}

	if (m_pFunctionName->GetType() != IrisIdentifierType::Constance && m_pFunctionName->GetType() != IrisIdentifierType::LocalVariable) {
		IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::IdenfierTypeIrregular, m_nLineNumber, pCompiler->GetCurrentFileIndex(), "Identifier of " + m_pFunctionName->GetIdentifierString() + " must be a LOCAL VARIABLE name.");
		return false;
	}
	
	if (m_pParameters && !m_pParameters->Ergodic([&](IrisExpression*& pExpression) -> bool {
		if (!pExpression->Accept(&ievvExpressionVisitor)) {
			return false;
		}
		return true;
	})) {
		return false;
	}

	if (m_pClosureBlock && !m_pClosureBlock->Accept(&isvvStatementVisitor)) {
		return false;
	}

	return true;
}
