#include "IrisComponents/IrisStatements/IrisFunctionStatement.h"
#include "IrisComponents/IrisParts/IrisFunctionHeader.h"
#include "IrisUnil/IrisBlock.h"
#include "IrisCompiler.h"
#include "IrisInstructorMaker.h"
#include "IrisUnil/IrisIdentifier.h"
#include "IrisFatalErrorHandler.h"
#include "IrisValidator/IrisStatementValidateVisitor.h"

#include "IrisThread/IrisThreadManager.h"

#include <list>
using namespace std;

bool IrisFunctionStatement::Generate()
{
	IrisCompiler* pCompiler = IrisCompiler::CurrentCompiler();
	IrisInstructorMaker* pMaker = IrisInstructorMaker::CurrentInstructor();
	pCompiler->SetLineNumber(m_nLineNumber);

	list<IR_DWORD> lsParameters;

	if (m_pFunctionHeader->m_pParameters) {
		if(!m_pFunctionHeader->m_pParameters->Ergodic(
			[&](IrisIdentifier*& pIdentifier) -> bool {
			lsParameters.push_back(pCompiler->GetIdentifierIndex(pIdentifier->GetIdentifierString(), pCompiler->GetCurrentFileIndex()));
			return true;
		}
		))
			return false;
	}
	
	IR_DWORD dwMethodNameIndex = pCompiler->GetIdentifierIndex(m_pFunctionHeader->m_pFunctionName->GetIdentifierString(), pCompiler->GetCurrentFileIndex());

	IR_DWORD dwVariablePrameterIndex = -1;
	if (m_pFunctionHeader->m_pVariableParameter) {
		dwVariablePrameterIndex = pCompiler->GetIdentifierIndex(m_pFunctionHeader->m_pVariableParameter->GetIdentifierString(), pCompiler->GetCurrentFileIndex());
	}
	IR_BYTE bWithCastBlock = m_pWithBlock ? 1 : 0;

	// 方法头
	pCompiler->IncreamDefineIndex();
	if (!m_pFunctionHeader->m_bIsClassMethod) {
		pMaker->imth_def(dwMethodNameIndex, lsParameters, dwVariablePrameterIndex, bWithCastBlock, pCompiler->GetDefineIndex());
	}
	else {
		pMaker->cmth_def(dwMethodNameIndex, lsParameters, dwVariablePrameterIndex, bWithCastBlock, pCompiler->GetDefineIndex());
	}

	if (m_pBlock) {
		// 方法体
		m_pBlock->Generate();
	}
	else {
		pCompiler->IncreamDefineIndex();
		pMaker->blk_def(pCompiler->GetDefineIndex());
		pMaker->end_def(pCompiler->GetDefineIndex());
		pCompiler->DecreamDefineIndex();
	}

	// 附加
	if (m_pWithBlock) {
		m_pWithBlock->Generate();
		m_pWithoutBlock->Generate();
	}

	pMaker->end_def(pCompiler->GetDefineIndex());
	pCompiler->DecreamDefineIndex();

	return true;
}

IrisFunctionStatement::IrisFunctionStatement(IrisFunctionHeader* pFunctionHeader, IrisBlock* pWithBlock, IrisBlock* pWithoutBlock, IrisBlock* pBlock) : m_pFunctionHeader(pFunctionHeader), m_pWithBlock(pWithBlock), m_pWithoutBlock(pWithoutBlock), m_pBlock(pBlock)
{
}


IrisFunctionStatement::~IrisFunctionStatement()
{
	if (m_pFunctionHeader)
		delete m_pFunctionHeader;
	if (m_pWithBlock)
		delete m_pWithBlock;
	if (m_pWithoutBlock)
		delete m_pWithoutBlock;
	if (m_pBlock)
		delete m_pBlock;
}

bool IrisFunctionStatement::Validate()
{
	IrisCompiler* pCompiler = IrisCompiler::CurrentCompiler();
	IrisStatementValidateVisitor isvvStatementVisitor;

	if (m_pFunctionHeader->m_pParameters) {
		if (!m_pFunctionHeader->m_pParameters->Ergodic(
			[&](IrisIdentifier*& pIdentifier) -> bool {
			if (pIdentifier->GetType() != IrisIdentifierType::LocalVariable) {
				IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::IdenfierTypeIrregular, m_nLineNumber, pCompiler->GetCurrentFileIndex(), "Identifier of " + pIdentifier->GetIdentifierString() + " must be a LOCAL VARIABLE name.", IrisThreadManager::CurrentThreadManager()->GetMainThreadInfo());
				return false;
			}
			return true;
		}
		))
			return false;
	}

	if (m_pFunctionHeader->m_pFunctionName->GetType() != IrisIdentifierType::LocalVariable
		&& m_pFunctionHeader->m_pFunctionName->GetType() != IrisIdentifierType::Constance) {
		IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::IdenfierTypeIrregular, m_nLineNumber, pCompiler->GetCurrentFileIndex(), "Identifier of " + m_pFunctionHeader->m_pFunctionName->GetIdentifierString() + " is NOT a valid method name.", IrisThreadManager::CurrentThreadManager()->GetMainThreadInfo());
		return false;
	}

	if (m_pFunctionHeader->m_pVariableParameter) {
		if (m_pFunctionHeader->m_pVariableParameter->GetType() != IrisIdentifierType::LocalVariable) {
			IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::IdenfierTypeIrregular, m_nLineNumber, pCompiler->GetCurrentFileIndex(), "Identifier of " + m_pFunctionHeader->m_pVariableParameter->GetIdentifierString() + " must be a LOCAL VARIABLE name.", IrisThreadManager::CurrentThreadManager()->GetMainThreadInfo());
			return false;
		}
	}

	pCompiler->PushUpperType(IrisCompiler::UpperType::MethodBlock);

	if (m_pBlock && !m_pBlock->Accept(&isvvStatementVisitor)) {
		return false;
	}

	if (m_pWithBlock && (!m_pBlock->Accept(&isvvStatementVisitor) || !m_pWithoutBlock->Accept(&isvvStatementVisitor))) {
		return false;
	}

	pCompiler->PopUpperType();

	return true;
}
