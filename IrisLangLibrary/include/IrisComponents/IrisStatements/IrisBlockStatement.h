#ifndef _H_IRISBLOCKSTATEMENT_
#define _H_IRISBLOCKSTATEMENT_

#include "IrisStatement.h"
class IrisBlockStatement :
	public IrisStatement
{
public:

	virtual bool Generate();

	IrisBlockStatement();
	virtual ~IrisBlockStatement();

	virtual bool Validate() override;
};

#endif